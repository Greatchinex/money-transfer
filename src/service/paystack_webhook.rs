use rust_decimal::Decimal;
use sea_orm::*;
use serde_json::{to_string, Value};
use thiserror::Error;
use tracing::{error, info};
use uuid::Uuid;

use crate::entities::{prelude::Wallets, sea_orm_active_enums::TrxType, transactions, wallets};
use crate::utils::paystack::verify_transaction;

#[derive(Error, Debug)]
pub enum WebhookHandlerError {
    #[error("Failed to make API request")]
    HttpRequestError(#[from] reqwest::Error),

    #[error("Database error occured")]
    DatabaseError(#[from] sea_orm::error::DbErr),
}

pub async fn handle_inflow_webhook(
    payload: &Value,
    db: &DatabaseConnection,
) -> Result<bool, WebhookHandlerError> {
    let reference = payload["data"]["reference"]
        .as_str()
        .unwrap_or_default()
        .to_string();

    let verify_trx = verify_transaction(&reference).await?;

    let n_unit: Decimal = 100.into();
    let status: &str = verify_trx["data"]["status"].as_str().unwrap_or_default();
    let amount: Decimal = verify_trx["data"]["amount"]
        .as_i64()
        .unwrap_or_default()
        .into();
    let provider_fees: Decimal = verify_trx["data"]["fees"]
        .as_i64()
        .unwrap_or_default()
        .into();

    if status != "success" || amount == 0.into() {
        return Ok(false);
    }

    let amt_in_naira = amount / n_unit;
    let user_id = verify_trx["data"]["metadata"]["user_id"]
        .as_str()
        .unwrap_or_default()
        .to_string();

    let txn = db.begin().await?;

    let my_wallet = Wallets::find()
        .filter(wallets::Column::UserId.eq(&user_id))
        .filter(wallets::Column::Default.eq(1))
        .one(&txn)
        .await;

    let my_wallet = match my_wallet {
        Ok(Some(my_wallet)) => my_wallet,
        Ok(None) => {
            info!("No wallet found for user {}", user_id);
            let _ = txn.rollback().await;
            return Ok(false);
        }
        Err(err) => {
            error!("DB error while trying to fetch user wallet: {}", err);
            let _ = txn.rollback().await;
            return Err(WebhookHandlerError::DatabaseError(err));
        }
    };

    let new_balance = my_wallet.current_balance + amt_in_naira;
    let previous_balance = my_wallet.current_balance;

    let mut wallet_model: wallets::ActiveModel = my_wallet.into();
    wallet_model.current_balance = Set(new_balance);
    wallet_model.previous_balance = Set(previous_balance);

    let wallet_model = match wallet_model.update(&txn).await {
        Ok(wallet) => wallet,
        Err(err) => {
            error!("DB error crediting user wallet: {}", err);
            let _ = txn.rollback().await;
            return Err(WebhookHandlerError::DatabaseError(err));
        }
    };

    let new_transaction = transactions::ActiveModel {
        uuid: Set(Uuid::new_v4().to_string()),
        amount: Set(amt_in_naira),
        trx_type: Set(Some(TrxType::Credit)),
        description: Set(String::from("Account funding")),
        provider_reference: Set(Some(reference)),
        current_balance: Set(new_balance),
        previous_balance: Set(previous_balance),
        user_id: Set(user_id),
        wallet_id: Set(wallet_model.uuid),
        provider: Set(String::from("paystack")),
        provider_fees: Set(provider_fees / n_unit),
        category: Set(String::from("funding")),
        meta: Set(Some(to_string(&payload).unwrap_or_default())),
        ..Default::default()
    };

    let _ = match new_transaction.insert(&txn).await {
        Ok(trx) => trx,
        Err(err) => {
            error!("DB error inserting new transaction: {}", err);
            let _ = txn.rollback().await;
            return Err(WebhookHandlerError::DatabaseError(err));
        }
    };

    let _ = txn.commit().await;

    Ok(true)
}
