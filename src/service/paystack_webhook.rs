use rust_decimal::Decimal;
use sea_orm::*;
use serde_json::Value;
use thiserror::Error;
use tracing::{error, info};
use uuid::Uuid;

use crate::entities::{
    prelude::Wallets,
    sea_orm_active_enums::{Status, TrxType},
    wallets,
};
use crate::utils::paystack::verify_transaction;
use crate::AppState;

use super::transaction_balance::{TransactionBalance, TransactionBalanceTrait, TrxCategory};

#[derive(Error, Debug)]
pub enum WebhookHandlerError {
    #[error("Failed to make API request")]
    HttpRequestError(#[from] reqwest::Error),

    #[error("Database error occured")]
    DatabaseError(#[from] sea_orm::error::DbErr),
}

pub async fn handle_inflow_webhook(
    payload: &Value,
    app_state: &AppState,
) -> Result<bool, WebhookHandlerError> {
    let reference = payload["data"]["reference"]
        .as_str()
        .unwrap_or_default()
        .to_string();

    let verify_trx = verify_transaction(&reference, &app_state.env).await?;

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

    let uuid = Uuid::new_v4();
    let amt_in_naira = amount / n_unit;
    let user_id = verify_trx["data"]["metadata"]["user_id"]
        .as_str()
        .unwrap_or_default()
        .to_string();

    let txn = app_state
        .db
        .begin_with_config(
            Some(IsolationLevel::RepeatableRead),
            Some(AccessMode::ReadWrite),
        )
        .await?;

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

    let save_trx = TransactionBalance {
        uuid: format!("{}", &uuid),
        amount: amt_in_naira,
        trx_type: TrxType::Credit,
        status: Status::Successful,
        description: format!("Funding of account. ID: {}", &uuid),
        provider_reference: Some(format!("{}", &reference)),
        current_balance: my_wallet.current_balance + amt_in_naira,
        previous_balance: my_wallet.current_balance,
        user_id: format!("{}", &user_id),
        wallet_id: format!("{}", &my_wallet.uuid),
        provider: format!("paystack"),
        fees: None,
        provider_fees: Some(provider_fees / n_unit),
        category: TrxCategory::Funding,
        meta: Some(payload.to_string()),
    };

    let _ = match save_trx.save_transaction_update_balance(&txn).await {
        Ok(resp) => resp,
        Err(err) => {
            error!("DB error updating wallet and creating transaction: {}", err);
            let _ = txn.rollback().await;
            return Err(WebhookHandlerError::DatabaseError(err));
        }
    };

    let _ = txn.commit().await;

    Ok(true)
}
