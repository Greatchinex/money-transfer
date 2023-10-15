use actix_web::{web, HttpResponse, Responder};
use sea_orm::*;
use serde_json::json;
use tracing::{error, instrument};

use crate::entities::{prelude::Wallets, users, wallets};
use crate::AppState;

#[instrument(skip(req_user, app_state), fields(user_id = %req_user.uuid))]
pub async fn my_wallets(
    req_user: web::ReqData<users::Model>,
    app_state: web::Data<AppState>,
) -> impl Responder {
    let wallets = Wallets::find()
        .filter(wallets::Column::UserId.eq(&req_user.uuid))
        .all(&app_state.db)
        .await;

    let wallets = match wallets {
        Ok(wallets) => wallets,
        Err(err) => {
            error!("Error retrieving user wallets: {}", err);
            return HttpResponse::InternalServerError()
                .json(json!({ "status": "error", "message": "Failed to fetch user wallets" }));
        }
    };

    HttpResponse::Ok().json(json!({
        "status": "success",
        "message": "Fetched user wallets",
        "data": { "wallets": wallets }
    }))
}
