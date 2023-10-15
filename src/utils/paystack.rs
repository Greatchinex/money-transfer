use reqwest::{header, Client, Error};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;

#[derive(Serialize, Deserialize, Debug)]
pub struct InitiateFundingResponse {
    status: bool,
    message: String,
    data: Option<InitiateFundingResponseData>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InitiateFundingResponseData {
    authorization_url: String,
    access_code: String,
    reference: String,
}

#[tokio::main]
pub async fn initiate_user_funding(
    email: &String,
    user_id: &String,
    amount: u64,
) -> Result<InitiateFundingResponse, Error> {
    let paystack_base_url =
        env::var("PAYSTACK_BASE_URL").expect("PAYSTACK_BASE_URL is not set in .env file");
    let paystack_secret =
        env::var("PAYSTACK_SECRET").expect("PAYSTACK_SECRET is not set in .env file");

    let client = Client::new();
    let url = format!("{paystack_base_url}/transaction/initialize");

    let response = client
        .post(&url)
        .json(&json!({
            "email": email,
            "amount": amount * 100,
            "metadata": { "user_id": user_id, "tokenized_charge": "false" }
        }))
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {}", paystack_secret))
        .send()
        .await?;

    let response_body = response.json::<InitiateFundingResponse>().await?;
    Ok(response_body)
}
