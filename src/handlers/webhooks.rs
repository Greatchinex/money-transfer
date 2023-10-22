use actix_web::{web, HttpRequest, HttpResponse, Responder};
use serde_json::{json, Value};
use tracing::{error, instrument};

use crate::service::paystack_webhook::handle_inflow_webhook;
use crate::utils::helpers::validate_signature;
use crate::AppState;

#[instrument(skip(req, app_state))]
pub async fn paystack_webhook(
    req: HttpRequest,
    body: web::Json<Value>,
    app_state: web::Data<AppState>,
) -> impl Responder {
    let signature = req
        .headers()
        .get("x-paystack-signature")
        .expect("x-paystack-signature not sent in header payload")
        .to_str()
        .expect("Failed to parse signaure header to string");

    // TODO: Properly look into below method
    let is_valid_signature = validate_signature(
        &body.to_string(),
        &signature.to_string(),
        &app_state.env.paystack_secret,
    );
    if !is_valid_signature {
        return HttpResponse::Ok()
            .json(json!({ "status": "success", "message": "Invlaid signature" }));
    }

    let event_type = body["event"].as_str().unwrap_or_default().to_string();

    if event_type == "charge.success" {
        let _ = match handle_inflow_webhook(&body, &app_state).await {
            Ok(bool) => bool,
            Err(err) => {
                error!("Error occured trying to fund user account: {}", err);
                return HttpResponse::BadRequest().json(json!({
                    "status": "error", "message": "Paystack webhook error"
                }));
            }
        };
    }

    HttpResponse::Ok()
        .json(json!({ "status": "success", "message": "Paystack webhook successful" }))
}
