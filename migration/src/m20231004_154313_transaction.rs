use sea_orm::EnumIter;
use sea_orm_migration::prelude::*;

use super::m20231003_223905_user::Users;
use super::m20231004_112043_wallet::Wallets;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Transactions::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Transactions::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(Transactions::Uuid)
                            .string()
                            .not_null()
                            .unique_key()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Transactions::Amount)
                            .decimal_len(18, 2)
                            .not_null()
                            .default(0.00),
                    )
                    .col(ColumnDef::new(Transactions::TrxType).enumeration(
                        TransactionType::Table,
                        [TransactionType::Credit, TransactionType::Debit],
                    ))
                    .col(
                        ColumnDef::new(Transactions::Status)
                            .enumeration(
                                TransactionStatus::Table,
                                [
                                    TransactionStatus::Successful,
                                    TransactionStatus::Pending,
                                    TransactionStatus::Failed,
                                ],
                            )
                            .default("successful"),
                    )
                    .col(
                        ColumnDef::new(Transactions::Description)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Transactions::ProviderReference)
                            .string()
                            .null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(Transactions::CurrentBalance)
                            .decimal_len(18, 2)
                            .not_null()
                            .default(0.00),
                    )
                    .col(
                        ColumnDef::new(Transactions::PreviousBalance)
                            .decimal_len(18, 2)
                            .not_null()
                            .default(0.00),
                    )
                    .col(ColumnDef::new(Transactions::UserId).string().not_null())
                    .col(ColumnDef::new(Transactions::WalletId).string().not_null())
                    .col(ColumnDef::new(Transactions::Provider).string().not_null())
                    .col(
                        ColumnDef::new(Transactions::Fees)
                            .decimal_len(18, 2)
                            .not_null()
                            .default(0.00),
                    )
                    .col(
                        ColumnDef::new(Transactions::ProviderFees)
                            .decimal_len(18, 2)
                            .not_null()
                            .default(0.00),
                    )
                    .col(ColumnDef::new(Transactions::Category).string().not_null())
                    .col(ColumnDef::new(Transactions::Meta).text().null())
                    .col(
                        ColumnDef::new(Transactions::CreatedAt)
                            .timestamp()
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Transactions::UpdatedAt)
                            .timestamp()
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .col(ColumnDef::new(Transactions::DeletedAt).timestamp().null())
                    .index(
                        Index::create()
                            .name("transactions_amount_index")
                            .col(Transactions::Amount),
                    )
                    .index(
                        Index::create()
                            .name("transactions_trx_type_index")
                            .col(Transactions::TrxType),
                    )
                    .index(
                        Index::create()
                            .name("transactions_status_index")
                            .col(Transactions::Status),
                    )
                    .index(
                        Index::create()
                            .name("transactions_user_id_index")
                            .col(Transactions::UserId),
                    )
                    .index(
                        Index::create()
                            .name("transactions_wallet_id_index")
                            .col(Transactions::WalletId),
                    )
                    .index(
                        Index::create()
                            .name("transactions_provider_index")
                            .col(Transactions::Provider),
                    )
                    .index(
                        Index::create()
                            .name("transactions_fees_index")
                            .col(Transactions::Fees),
                    )
                    .index(
                        Index::create()
                            .name("transactions_category_index")
                            .col(Transactions::Category),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("transactions_user_id_foreign")
                            .from(Transactions::Table, Transactions::UserId)
                            .to(Users::Table, Users::Uuid),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("transactions_wallet_id_foreign")
                            .from(Transactions::Table, Transactions::WalletId)
                            .to(Wallets::Table, Wallets::Uuid),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Transactions::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Transactions {
    Table,
    Id,
    Uuid,
    Amount,
    TrxType,
    Status,
    Description,
    ProviderReference,
    CurrentBalance,
    PreviousBalance,
    UserId,
    WalletId,
    Provider,
    Fees,
    ProviderFees,
    Category,
    Meta,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
}

#[derive(Iden, EnumIter)]
pub enum TransactionStatus {
    Table,
    Successful,
    Pending,
    Failed,
}

#[derive(Iden, EnumIter)]
pub enum TransactionType {
    Table,
    Credit,
    Debit,
}
