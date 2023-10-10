use actix_web::{post, web, HttpResponse, Responder};
use argonautica::Hasher;
use sea_orm::*;
use serde_json::json;
use std::env;
use uuid::Uuid;
use validator::Validate;

use crate::dto::users::SignupBody;
use crate::entities::{prelude::Users, users};
use crate::AppState;

#[post("/signup")]
pub async fn signup(body: web::Json<SignupBody>, app_state: web::Data<AppState>) -> impl Responder {
    let user_payload = body.into_inner();
    let validate_payload = user_payload.validate();
    if validate_payload.is_err() {
        return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "Validation errors",
            "data": validate_payload.unwrap_err().errors(),
        }));
    }

    let lowercase_email = user_payload.email.to_lowercase();
    let check_user = Users::find()
        .filter(users::Column::Email.eq(&lowercase_email))
        .one(&app_state.db)
        .await
        .expect("Could not get user");

    if let Some(_) = check_user {
        return HttpResponse::NotFound()
            .json(json!({ "status": "error",  "message": "User with this email already exists" }));
    }

    let hash_key = env::var("HASH_KEY").expect("HASH_KEY is not set in .env file");
    let mut hasher = Hasher::default();
    let hashed_password = hasher
        .with_password(user_payload.password)
        .with_secret_key(hash_key)
        .hash()
        .expect("Failed to hash user password");

    let new_user = users::ActiveModel {
        uuid: Set(Uuid::new_v4().to_string()),
        first_name: Set(user_payload.first_name),
        last_name: Set(user_payload.last_name),
        email: Set(lowercase_email),
        password: Set(hashed_password),
        ..Default::default()
    };

    new_user
        .insert(&app_state.db)
        .await
        .expect("Failed to create user");

    // TODO: Handle email delivery for user verification

    HttpResponse::Created()
        .json(json!({ "status": "success", "message": "User created successfully" }))
}

#[post("/login")]
pub async fn login() -> impl Responder {
    HttpResponse::Ok().json(json!({ "status": "success", "message": "Login successful" }))
}
