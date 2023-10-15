use actix_web::{web, HttpRequest, HttpResponse, Responder};
use serde_json::{json, Value};
use std::env;
use tracing::instrument;

use crate::utils::helpers::validate_signature;

#[instrument(skip(req))]
pub async fn paystack_webhook(req: HttpRequest, body: web::Json<Value>) -> impl Responder {
    let paystack_secret =
        env::var("PAYSTACK_SECRET").expect("PAYSTACK_SECRET is not set in .env file");
    let signature = req
        .headers()
        .get("x-paystack-signature")
        .expect("x-paystack-signature not sent in header payload")
        .to_str()
        .expect("Failed to parse signaure header to string");

    let is_valid_signature =
        validate_signature(&body.to_string(), &signature.to_string(), &paystack_secret);

    if !is_valid_signature {
        return HttpResponse::Ok()
            .json(json!({ "status": "success", "message": "Invlaid signature" }));
    }

    // TODO: Call function to handle validation and crediting wallet

    HttpResponse::Ok()
        .json(json!({ "status": "success", "message": "Paystack webhook successful" }))
}
