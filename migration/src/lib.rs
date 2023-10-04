pub use sea_orm_migration::prelude::*;

mod m20231003_223905_user;
mod m20231004_112043_wallet;
mod m20231004_154313_transaction;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20231003_223905_user::Migration),
            Box::new(m20231004_112043_wallet::Migration),
            Box::new(m20231004_154313_transaction::Migration),
        ]
    }
}
