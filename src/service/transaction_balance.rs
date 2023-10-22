use async_trait::async_trait;
use chrono::Utc;
use rust_decimal::Decimal;
use sea_orm::*;

use crate::entities::{
    prelude::Wallets,
    sea_orm_active_enums::{Status, TrxType},
    transactions, wallets,
};

#[derive(Debug)]
pub enum TrxCategory {
    P2P,
    Funding,
    Outward,
}

pub struct TransactionBalance {
    pub uuid: String,
    pub amount: Decimal,
    pub trx_type: TrxType,
    pub status: Status,
    pub description: String,
    pub provider_reference: Option<String>,
    pub current_balance: Decimal,
    pub previous_balance: Decimal,
    pub user_id: String,
    pub wallet_id: String,
    pub provider: String,
    pub fees: Option<Decimal>,
    pub provider_fees: Option<Decimal>,
    pub category: TrxCategory,
    pub meta: Option<String>,
}

#[async_trait]
pub trait TransactionBalanceTrait {
    async fn save_transaction_update_balance(self, txn: &DatabaseTransaction) -> Result<(), DbErr>;
}

impl TrxCategory {
    fn to_string(&self) -> String {
        match self {
            TrxCategory::Funding => format!("funding"),
            TrxCategory::P2P => format!("p2p"),
            TrxCategory::Outward => format!("outward"),
        }
    }
}

#[async_trait]
impl TransactionBalanceTrait for TransactionBalance {
    async fn save_transaction_update_balance(self, txn: &DatabaseTransaction) -> Result<(), DbErr> {
        let mut update_balance = sea_query::Query::update();
        update_balance
            .table(Wallets.table_name().into_identity())
            .values([
                (wallets::Column::CurrentBalance, self.current_balance.into()),
                (
                    wallets::Column::PreviousBalance,
                    self.previous_balance.into(),
                ),
                (wallets::Column::UpdatedAt, Utc::now().into()),
            ])
            .and_where(sea_query::Expr::col(wallets::Column::Uuid).eq(&self.wallet_id));

        let wallet_stmt = txn.get_database_backend().build(&update_balance);
        let _ = txn.execute(wallet_stmt).await?;

        let new_transaction = transactions::ActiveModel {
            uuid: Set(self.uuid),
            amount: Set(self.amount),
            trx_type: Set(Some(self.trx_type)),
            status: Set(Some(self.status)),
            description: Set(self.description),
            provider_reference: Set(Some(self.provider_reference.unwrap_or_default())),
            current_balance: Set(self.current_balance),
            previous_balance: Set(self.previous_balance),
            user_id: Set(self.user_id),
            wallet_id: Set(self.wallet_id),
            provider: Set(self.provider),
            fees: Set(self.fees.unwrap_or_default()),
            provider_fees: Set(self.provider_fees.unwrap_or_default()),
            category: Set(self.category.to_string()),
            meta: Set(Some(self.meta.unwrap_or_default())),
            ..Default::default()
        };

        let _ = new_transaction.insert(txn).await?;

        Ok(())
    }
}
