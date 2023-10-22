use std::env::var;

#[derive(Clone, Debug)]
pub struct EnvConfig {
    pub app_name: String,
    pub port: String,
    pub host: String,
    pub database_url: String,
    pub app_key: String,
    pub hash_key: String,
    pub app_base_url: String,
    pub smtp_provider: String,
    pub smtp_user: String,
    pub smtp_key: String,
    pub from_email: String,
    pub paystack_base_url: String,
    pub paystack_secret: String,
}

impl EnvConfig {
    pub fn init() -> EnvConfig {
        EnvConfig {
            app_name: var("APP_NAME").unwrap_or(String::from("money-transfer-v1")),
            port: var("PORT").expect("Missing env PORT"),
            host: var("HOST").expect("Missing env HOST"),
            database_url: var("DATABASE_URL").expect("Missing env DATABASE_URL"),
            app_key: var("APP_KEY").expect("Missing env APP_KEY"),
            hash_key: var("HASH_KEY").expect("Missing env HASH_KEY"),
            app_base_url: var("APP_BASE_URL").expect("Missing env APP_BASE_URL"),
            smtp_provider: var("SMTP_PROVIDER").unwrap_or_default(),
            smtp_user: var("SMTP_USER").unwrap_or_default(),
            smtp_key: var("SMTP_KEY").unwrap_or_default(),
            from_email: var("FROM_EMAIL").unwrap_or(String::from("support@moneytransfer.am")),
            paystack_base_url: var("PAYSTACK_BASE_URL")
                .unwrap_or(String::from("https://api.paystack.co")),
            paystack_secret: var("PAYSTACK_SECRET").expect("Missing env PAYSTACK_SECRET"),
        }
    }
}
