use sea_orm_migration::prelude::*;

use super::m20231003_223905_user::Users;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Wallets::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Wallets::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(Wallets::Uuid)
                            .string()
                            .not_null()
                            .unique_key()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Wallets::Default)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(Wallets::CurrentBalance)
                            .decimal_len(18, 2)
                            .not_null()
                            .default(0.00),
                    )
                    .col(
                        ColumnDef::new(Wallets::PreviousBalance)
                            .decimal_len(18, 2)
                            .not_null()
                            .default(0.00),
                    )
                    .col(ColumnDef::new(Wallets::UserId).string().not_null())
                    .col(
                        ColumnDef::new(Wallets::CreatedAt)
                            .timestamp()
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Wallets::UpdatedAt)
                            .timestamp()
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .col(ColumnDef::new(Wallets::DeletedAt).timestamp().null())
                    .index(
                        Index::create()
                            .unique()
                            .name("wallets_user_id_index")
                            .col(Wallets::UserId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("wallets_user_id_foreign")
                            .from(Wallets::Table, Wallets::UserId)
                            .to(Users::Table, Users::Uuid),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Wallets::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Wallets {
    Table,
    Id,
    Uuid,
    Default,
    CurrentBalance,
    PreviousBalance,
    UserId,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
}
