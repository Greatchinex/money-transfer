use argonautica::Verifier;
use std::env;
use tracing::error;

pub fn validate_password(hashed_password: &String, compare_password: &String) -> bool {
    let hash_key = env::var("HASH_KEY").expect("HASH_KEY is not set in .env file");

    let mut verifier = Verifier::default();
    verifier
        .with_hash(hashed_password)
        .with_password(compare_password)
        .with_secret_key(hash_key)
        .verify()
        .unwrap_or_else(|err| {
            error!("Failed to verify user password hash ===> {}", err);
            false
        })
}
