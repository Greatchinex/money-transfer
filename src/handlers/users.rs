use actix_web::{post, web, HttpResponse, Responder};
use argonautica::{Hasher, Verifier};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use sea_orm::*;
use serde_json::json;
use std::env;
use tracing::{error, instrument};
use uuid::Uuid;
use validator::Validate;

use crate::dto::users::{LoginBody, SignupBody, TokenClaims};
use crate::entities::{prelude::Users, users};
use crate::AppState;

#[post("/signup")]
#[instrument(skip(body, app_state), fields(user_email = %body.email))]
pub async fn signup(body: web::Json<SignupBody>, app_state: web::Data<AppState>) -> impl Responder {
    let user_payload = match body.validate() {
        Ok(_) => body.into_inner(),
        Err(err) => {
            return HttpResponse::BadRequest()
                .json(json!({ "status": "error", "message": "Validation errors", "data": err }));
        }
    };

    let lowercase_email = user_payload.email.to_lowercase();
    let check_user = Users::find()
        .filter(users::Column::Email.eq(&lowercase_email))
        .one(&app_state.db)
        .await;

    let check_user = match check_user {
        Ok(check_user) => check_user,
        Err(err) => {
            error!("Database error while trying to fetch a user ===> {}", err);
            return HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "An error occured trying to validate user"
            }));
        }
    };

    if let Some(_) = check_user {
        return HttpResponse::BadRequest()
            .json(json!({ "status": "error",  "message": "User with this email already exists" }));
    }

    let hash_key = env::var("HASH_KEY").expect("HASH_KEY is not set in .env file");
    let mut hasher = Hasher::default();
    let hashed_password = hasher
        .with_password(user_payload.password)
        .with_secret_key(hash_key)
        .hash();

    let hashed_password = match hashed_password {
        Ok(hashed_password) => hashed_password,
        Err(err) => {
            error!("Failed to hash password ===> {}", err);
            return HttpResponse::InternalServerError()
                .json(json!({ "status": "error", "message": "An unexpected error occured" }));
        }
    };

    let new_user = users::ActiveModel {
        uuid: Set(Uuid::new_v4().to_string()),
        first_name: Set(user_payload.first_name),
        last_name: Set(user_payload.last_name),
        email: Set(lowercase_email),
        password: Set(hashed_password),
        ..Default::default()
    };

    let saved_user = new_user.insert(&app_state.db).await;
    if let Err(err) = saved_user {
        error!("Database error when trying to save user ===> {}", err);
        return HttpResponse::InternalServerError().json(
            json!({ "status": "error", "message": "An error occured trying to create user" }),
        );
    }

    // TODO: Handle email delivery for user verification

    HttpResponse::Created()
        .json(json!({ "status": "success", "message": "User created successfully" }))
}

#[post("/login")]
#[instrument(skip(body, app_state), fields(user_email = %body.email))]
pub async fn login(body: web::Json<LoginBody>, app_state: web::Data<AppState>) -> impl Responder {
    let user_payload = match body.validate() {
        Ok(_) => body.into_inner(),
        Err(err) => {
            return HttpResponse::BadRequest()
                .json(json!({ "status": "error", "message": "Validation errors", "data": err }));
        }
    };

    let lowercase_email = user_payload.email.to_lowercase();
    let check_user = Users::find()
        .filter(users::Column::Email.eq(&lowercase_email))
        .one(&app_state.db)
        .await;

    let check_user = match check_user {
        Ok(Some(check_user)) => check_user,
        Ok(None) => {
            return HttpResponse::BadRequest()
                .json(json!({ "status": "error", "message": "Incorrect login details" }));
        }
        Err(err) => {
            error!("Database error while validating user details ===> {}", err);
            return HttpResponse::InternalServerError().json(
                json!({ "status": "error", "message": "An error occured trying to validate user" }),
            );
        }
    };

    let hash_key = env::var("HASH_KEY").expect("HASH_KEY is not set in .env file");
    let mut verifier = Verifier::default();
    let is_valid_password = verifier
        .with_hash(&check_user.password)
        .with_password(user_payload.password)
        .with_secret_key(hash_key)
        .verify()
        .unwrap_or_else(|err| {
            error!("Failed to verify user password hash ===> {}", err);
            false
        });

    if !is_valid_password {
        return HttpResponse::BadRequest()
            .json(json!({ "status": "error",  "message": "Incorrect login details" }));
    }

    let token_secret = env::var("APP_KEY").expect("APP_KEY is not set in .env file");
    let now = Utc::now();
    let claims = TokenClaims {
        sub: check_user.uuid.to_string(),
        exp: (now + Duration::minutes(60)).timestamp() as usize,
        iat: now.timestamp() as usize,
    };

    let token = match encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(token_secret.as_ref()),
    ) {
        Ok(token) => token,
        Err(err) => {
            error!("Failed to sign token ===> {}", err);
            return HttpResponse::InternalServerError()
                .json(json!({ "status": "error", "message": "An unexpected error occured" }));
        }
    };

    HttpResponse::Ok().json(json!({
        "status": "success",
        "message": "Login successful",
        "data": {
            "token": token,
            "user": check_user.filter_response()
        }
    }))
}
