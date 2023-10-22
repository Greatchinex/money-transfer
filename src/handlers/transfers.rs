use actix_web::{web, HttpResponse, Responder};
use sea_orm::*;
use serde_json::json;
use tracing::{error, instrument};
use validator::Validate;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::dto::transfers::{ InitiateFundingBody, P2PTransferBody };
use crate::entities::{ prelude::{ Users, Wallets }, users, wallets, sea_orm_active_enums::{ TrxType, Status } };
use crate::utils::helpers::validate_password;
use crate::utils::paystack::initiate_user_funding;
use crate::service::transaction_balance::{ TransactionBalance, TrxCategory, TransactionBalanceTrait };
use crate::AppState;

#[instrument(skip(body, req_user), fields(user_id = %req_user.uuid, amount = %body.amount))]
pub async fn fund_account(
    body: web::Json<InitiateFundingBody>,
    req_user: web::ReqData<users::Model>,
) -> impl Responder {
    let request_payload = match body.validate() {
        Ok(_) => body.into_inner(),
        Err(err) => {
            return HttpResponse::BadRequest().json(json!({
                "status": "error", "message": "Validation errors", "data": err
            }));
        }
    };

    if req_user.is_verified != 1 {
        return HttpResponse::BadRequest()
            .json(json!({ "status": "error",  "message": "Please verify your account before taking this action" }));
    }

    let is_valid_password = validate_password(&req_user.password, &request_payload.password);
    if !is_valid_password {
        return HttpResponse::BadRequest()
            .json(json!({ "status": "error",  "message": "Wrong password provided" }));
    }

    let response = initiate_user_funding(&req_user.email, &req_user.uuid, request_payload.amount).await;

    match response {
        Ok(response) => {
            return HttpResponse::Ok().json(json!({
                "status": "success",
                "message": "Funding initiated successfully",
                "data": response.data
            }));
        }
        Err(err) => {
            error!("Error initiating user funding ===> {}", err);
            return HttpResponse::BadRequest().json(json!({ 
                "status": "error",  "message": "Cannot initiate funding at this time, Please try again later or contact support"
            }));
        }
    }
}

#[instrument(skip(body, req_user, app_state), fields(user_id = %req_user.uuid, amount = %body.amount, receiver_id = %body.receiver_id))]
pub async fn p2p_transfer(
    body: web::Json<P2PTransferBody>, 
    req_user: web::ReqData<users::Model>, 
    app_state: web::Data<AppState>
) -> impl Responder {
    let request_payload = match body.validate() {
        Ok(_) => body.into_inner(),
        Err(err) => {
            return HttpResponse::BadRequest().json(json!({
                "status": "error", "message": "Validation errors", "data": err
            }));
        }
    };

    if req_user.is_verified != 1 {
        return HttpResponse::BadRequest()
            .json(json!({ "status": "error",  "message": "Please verify your account before taking this action" }));
    }

    if req_user.uuid == request_payload.receiver_id {
        return HttpResponse::BadRequest()
            .json(json!({ "status": "error",  "message": "Cannot send funds to yourself" }));
    }

    let hashed_pin = match &req_user.withdrawal_pin {
        Some(pin) => pin,
        None => {
            return HttpResponse::BadRequest()
                .json(json!({ "status": "error",  "message": "Please set your pin before taking this action" }));
        }
    };

    let is_valid_pin = validate_password(hashed_pin, &request_payload.pin);
    if !is_valid_pin {
        return HttpResponse::BadRequest()
            .json(json!({ "status": "error",  "message": "Incorrect PIN" }));
    }

    let receiver = Users::find()
        .filter(users::Column::Uuid.eq(&request_payload.receiver_id))
        .one(&app_state.db)
        .await;

    let receiver = match receiver {
        Ok(Some(receiver)) => receiver,
        Ok(None) => {
            return HttpResponse::BadRequest()
                .json(json!({ "status": "error", "message": "User you are trying to send funds to not found" }));
        }
        Err(err) => {
            error!("DB error validating receiver details ===> {}", err);
            return HttpResponse::InternalServerError().json(
                json!({ "status": "error", "message": "An error occured trying to validate receiver" }),
            );
        }
    };

    let sender_name = format!("{} {}", req_user.last_name, req_user.first_name);
    let receiver_name = format!("{} {}", receiver.last_name, receiver.first_name);

    if receiver.is_verified != 1 {
        let msg = format!("Cannot send funds to {}, As they are not yet verified", &receiver_name);
        return HttpResponse::BadRequest()
            .json(json!({ "status": "error",  "message": msg }));
    }

    let amount: Decimal = request_payload.amount.into();
    let txn = app_state.db
        .begin_with_config(Some(IsolationLevel::RepeatableRead), Some(AccessMode::ReadWrite))
        .await
        .expect("Failed to start a DB transaction");

    let user_wallets = Wallets::find()
        .filter(
            Condition::any()
            .add(wallets::Column::UserId.eq(&req_user.uuid).and(wallets::Column::Default.eq(1)))
            .add(wallets::Column::UserId.eq(&receiver.uuid).and(wallets::Column::Default.eq(1)))
        )
        .all(&txn)
        .await;
        
    let user_wallets = match user_wallets {
        Ok(user_wallets) => user_wallets,
        Err(err) => {
            error!("DB error fetching wallets ===> {}", err);
            let _ = txn.rollback().await;
            return HttpResponse::InternalServerError().json(
                json!({ "status": "error", "message": "An unexpected error occured" }),
            );
        }
    };

    let sender_wallet = match user_wallets.iter().find(|wallet| wallet.user_id == req_user.uuid) {
        Some(sender_wallet) => sender_wallet.to_owned(),
        None => {
            let _ = txn.rollback().await;
            return HttpResponse::BadRequest().json(
                json!({ "status": "error", "message": "You do not seem to have a valid wallet yet. Please contact support" }),
            );
        }
    };

    if amount > sender_wallet.current_balance {
        let _ = txn.rollback().await;
        return HttpResponse::BadRequest().json(
            json!({ "status": "error", "message": "Insufficient Funds" }),
        );
    }

    let receiver_wallet = match user_wallets.iter().find(|wallet| wallet.user_id == receiver.uuid) {
        Some(receiver_wallet) => receiver_wallet.to_owned(),
        None => {
            let msg = format!("{} Does not have a valid wallet yet", &receiver_name);
            let _ = txn.rollback().await;
            return HttpResponse::BadRequest().json(
                json!({ "status": "error", "message": msg }),
            );
        }
    };

    if sender_wallet.uuid == receiver_wallet.uuid {
        let _ = txn.rollback().await;
        return HttpResponse::BadRequest()
            .json(json!({ "status": "error",  "message": "Cannot send funds to the same wallet" }));
    }

    let sender_ref = Uuid::new_v4();
    let receiver_ref = Uuid::new_v4();
    let narration = request_payload.narration.unwrap_or(String::from("Wallet Transfer"));

    let meta = json!({
        "sender_name": &sender_name,
        "receiver_name": &receiver_name,
        "sender_wallet_id": &sender_wallet.uuid,
        "receiver_wallet_id": &receiver_wallet.uuid
    }).to_string();


    let debit_sender = TransactionBalance {
        uuid: format!("{}", &sender_ref),
        amount,
        trx_type: TrxType::Debit,
        status: Status::Successful,
        description: format!("{} - TO {}", &narration, &receiver_name),
        provider_reference: Some(format!("{}", &receiver_ref)),
        current_balance: sender_wallet.current_balance - amount,
        previous_balance: sender_wallet.current_balance,
        user_id: format!("{}", &sender_wallet.user_id),
        wallet_id: format!("{}", &sender_wallet.uuid),
        provider: format!("money-transfer"),
        fees: None,
        provider_fees: None,
        category: TrxCategory::P2P,
        meta: Some(format!("{}", &meta)),
    };

    let _ = match debit_sender.save_transaction_update_balance(&txn).await {
        Ok(resp) => resp,
        Err(err) => {
            error!("DB error debiting sender: {}", err);
            let _ = txn.rollback().await;
            return HttpResponse::BadRequest().json(
                json!({ "status": "error", "message": "Transfer Error" }),
            );
        }
    };

    let credit_receiver = TransactionBalance {
        uuid: format!("{}", &receiver_ref),
        amount,
        trx_type: TrxType::Credit,
        status: Status::Successful,
        description: format!("{} - FROM {}", &narration, &sender_name),
        provider_reference: Some(format!("{}", &sender_ref)),
        current_balance: receiver_wallet.current_balance + amount,
        previous_balance: receiver_wallet.current_balance,
        user_id: format!("{}", &receiver_wallet.user_id),
        wallet_id: format!("{}", &receiver_wallet.uuid),
        provider: format!("money-transfer"),
        fees: None,
        provider_fees: None,
        category: TrxCategory::P2P,
        meta: Some(format!("{}", &meta)),
    };

    let _ = match credit_receiver.save_transaction_update_balance(&txn).await {
        Ok(resp) => resp,
        Err(err) => {
            error!("DB error crediting receiver: {}", err);
            let _ = txn.rollback().await;
            return HttpResponse::BadRequest().json(
                json!({ "status": "error", "message": "Error trying to send funds to receiver" }),
            );
        }
    };

    let _ = txn.commit().await;

    HttpResponse::Created()
        .json(json!({ "status": "success", "message": "Funds sent successfully" }))
}
