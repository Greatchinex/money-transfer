use actix_web::{web, HttpResponse, Responder};
use serde_json::json;
use tracing::{error, instrument};
use validator::Validate;

use crate::dto::transfers::InitiateFundingBody;
use crate::entities::users;
use crate::utils::helpers::validate_password;
use crate::utils::paystack::initiate_user_funding;

#[instrument(skip(body, req_user), fields(user_id = %req_user.uuid, amount = %body.amount))]
pub async fn fund_account(
    body: web::Json<InitiateFundingBody>,
    req_user: web::ReqData<users::Model>,
) -> impl Responder {
    let request_payload = match body.validate() {
        Ok(_) => body.into_inner(),
        Err(err) => {
            return HttpResponse::BadRequest().json(json!({
                "status": "error", "message": "Validation errors", "data": err
            }));
        }
    };

    if req_user.is_verified != 1 {
        return HttpResponse::BadRequest()
            .json(json!({ "status": "error",  "message": "Please verify your account before taking this action" }));
    }

    let is_valid_password = validate_password(&req_user.password, &request_payload.password);
    if !is_valid_password {
        return HttpResponse::BadRequest()
            .json(json!({ "status": "error",  "message": "Wrong password provided" }));
    }

    let response = initiate_user_funding(&req_user.email, &req_user.uuid, request_payload.amount).await;

    match response {
        Ok(response) => {
            return HttpResponse::Ok().json(json!({
                "status": "success",
                "message": "Funding initiated successfully",
                "data": response.data
            }));
        }
        Err(err) => {
            error!("Error initiating user funding ===> {}", err);
            return HttpResponse::BadRequest().json(json!({ 
                "status": "error",  "message": "Cannot initiate funding at this time, Please try again later or contact support"
            }));
        }
    }
}
