use argonautica::Verifier;
use hex;
use ring::hmac;
use tracing::error;

pub fn validate_password(
    hashed_password: &String,
    compare_password: &String,
    hash_key: &String,
) -> bool {
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

pub fn validate_signature(payload: &String, signature: &String, secret: &String) -> bool {
    let key = hmac::Key::new(hmac::HMAC_SHA512, secret.as_bytes());
    let calc_signature = hmac::sign(&key, payload.as_bytes());
    let hash = hex::encode(calc_signature.as_ref());

    hash == signature.to_string()
}
