use reqwest::{header, Client, Error};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env;

#[derive(Serialize, Deserialize, Debug)]
pub struct InitiateFundingResponse {
    pub status: bool,
    pub message: String,
    pub data: Option<InitiateFundingResponseData>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InitiateFundingResponseData {
    pub authorization_url: String,
    pub access_code: String,
    pub reference: String,
}

pub async fn initiate_user_funding(
    email: &String,
    user_id: &String,
    amount: u64,
) -> Result<InitiateFundingResponse, Error> {
    let base_url =
        env::var("PAYSTACK_BASE_URL").expect("PAYSTACK_BASE_URL is not set in .env file");
    let secret = env::var("PAYSTACK_SECRET").expect("PAYSTACK_SECRET is not set in .env file");
    let url = format!("{base_url}/transaction/initialize");

    let client = Client::new();
    let response = client
        .post(&url)
        .json(&json!({
            "email": email,
            "amount": amount * 100,
            "metadata": { "user_id": user_id, "tokenized_charge": "false" }
        }))
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {}", secret))
        .send()
        .await?;

    let response_body = response.json::<InitiateFundingResponse>().await?;
    Ok(response_body)
}

pub async fn verify_transaction(reference: &String) -> Result<Value, Error> {
    let base_url =
        env::var("PAYSTACK_BASE_URL").expect("PAYSTACK_BASE_URL is not set in .env file");
    let secret = env::var("PAYSTACK_SECRET").expect("PAYSTACK_SECRET is not set in .env file");
    let url = format!("{base_url}/transaction/verify/{reference}");

    let client = Client::new();
    let response = client
        .get(&url)
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {}", secret))
        .send()
        .await?;

    let response_body = response.json::<Value>().await?;
    Ok(response_body)
}
