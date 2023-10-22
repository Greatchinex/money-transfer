use actix_web::{http, web, HttpResponse, Responder};
use argonautica::Hasher;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use sea_orm::*;
use serde_json::json;
use tracing::{error, instrument};
use uuid::Uuid;
use validator::Validate;

use crate::dto::users::{
    LoginBody, SetWithdrawalPinBody, SignupBody, TokenClaims, VerifyAccountParams,
};
use crate::entities::{prelude::Users, users, wallets};
use crate::utils::{
    email_template::verify_account_template,
    helpers::validate_password,
    send_email::{SendEmail, SendEmailTrait},
};
use crate::AppState;

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

    let _ = match check_user {
        Ok(None) => true,
        Ok(Some(_)) => {
            return HttpResponse::BadRequest().json(
                json!({ "status": "error",  "message": "User with this email already exists" }),
            );
        }
        Err(err) => {
            error!("Database error while trying to fetch a user ===> {}", err);
            return HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "An error occured trying to validate user"
            }));
        }
    };

    let mut hasher = Hasher::default();
    let hashed_password = hasher
        .with_password(user_payload.password)
        .with_secret_key(&app_state.env.hash_key)
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
        first_name: Set(format!("{}", &user_payload.first_name)),
        last_name: Set(format!("{}", &user_payload.last_name)),
        email: Set(format!("{}", &lowercase_email)),
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

    // SIGN TOKEN FOR EMAIL VERIFICATION
    let now = Utc::now();
    let claims = TokenClaims {
        sub: format!("{}", &lowercase_email),
        auth_type: String::from("ACCOUNT_VERIFICATION"),
        exp: (now + Duration::days(3)).timestamp() as usize,
        iat: now.timestamp() as usize,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(app_state.env.app_key.as_ref()),
    )
    .unwrap_or_else(|err| {
        error!("Error signing verification token: {}", err);
        String::new()
    });

    let template = verify_account_template(&user_payload.first_name, &token, &app_state.env);
    let email = SendEmail {
        to: claims.sub,
        from: format!("{}", &app_state.env.from_email),
        subject: String::from("WELCOME, VERIFY YOUR ACCOUNT"),
        template,
    };

    let _ = email.send_email(&app_state.env).await;

    HttpResponse::Created()
        .json(json!({ "status": "success", "message": "User created successfully" }))
}

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

    let is_valid_password = validate_password(
        &check_user.password,
        &user_payload.password,
        &app_state.env.hash_key,
    );
    if !is_valid_password {
        return HttpResponse::BadRequest()
            .json(json!({ "status": "error",  "message": "Incorrect login details" }));
    }

    let now = Utc::now();
    let claims = TokenClaims {
        sub: check_user.uuid.to_string(),
        auth_type: String::from("USER_AUTH"),
        exp: (now + Duration::minutes(60)).timestamp() as usize,
        iat: now.timestamp() as usize,
    };

    let token = match encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(app_state.env.app_key.as_ref()),
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

#[instrument(skip(req_user), fields(user_id = %req_user.uuid))]
pub async fn me(req_user: web::ReqData<users::Model>) -> impl Responder {
    HttpResponse::Ok().json(json!({
        "status": "success",
        "message": "User fetched successfully",
        "data": {
            "user": req_user.filter_response()
        }
    }))
}

// NOTE: Ideally there should be an error and success page created by a frontend dev to redirect a user on success/failure
// I am redirecting to the project repo on success and my github profile page on error
pub async fn verify_account(
    query: web::Query<VerifyAccountParams>,
    app_state: web::Data<AppState>,
) -> impl Responder {
    let claims = match decode::<TokenClaims>(
        &query.token,
        &DecodingKey::from_secret(app_state.env.app_key.as_ref()),
        &Validation::default(),
    ) {
        Ok(c) => c.claims,
        Err(err) => {
            error!("Error decoding verification token ===> {}", err);
            return HttpResponse::Found()
                .insert_header((http::header::LOCATION, "https://github.com/Greatchinex"))
                .finish();
        }
    };

    if claims.auth_type != "ACCOUNT_VERIFICATION" {
        return HttpResponse::Found()
            .insert_header((http::header::LOCATION, "https://github.com/Greatchinex"))
            .finish();
    }

    let check_user = Users::find()
        .filter(users::Column::Email.eq(&claims.sub))
        .one(&app_state.db)
        .await;

    let check_user = match check_user {
        Ok(Some(check_user)) => check_user,
        Ok(None) => {
            return HttpResponse::Found()
                .insert_header((http::header::LOCATION, "https://github.com/Greatchinex"))
                .finish()
        }
        Err(err) => {
            error!("Fetch user DB error ===> {}", err);
            return HttpResponse::Found()
                .insert_header((http::header::LOCATION, "https://github.com/Greatchinex"))
                .finish();
        }
    };

    if check_user.is_verified == 1 {
        return HttpResponse::Found()
            .insert_header((http::header::LOCATION, "https://github.com/Greatchinex"))
            .finish();
    }

    let user_id = format!("{}", &check_user.uuid);
    let mut user: users::ActiveModel = check_user.into();

    let txn = app_state
        .db
        .begin_with_config(
            Some(IsolationLevel::RepeatableRead),
            Some(AccessMode::ReadWrite),
        )
        .await
        .expect("Failed to start a DB transaction");

    user.is_verified = Set(1);
    user.updated_at = Set(Utc::now());
    let _ = match user.update(&txn).await {
        Ok(user) => user,
        Err(err) => {
            error!("Failed to update user status for {}: {}", &claims.sub, err);
            let _ = txn.rollback().await;
            return HttpResponse::Found()
                .insert_header((http::header::LOCATION, "https://github.com/Greatchinex"))
                .finish();
        }
    };

    let new_wallet = wallets::ActiveModel {
        uuid: Set(format!("{}", Uuid::new_v4())),
        user_id: Set(user_id),
        ..Default::default()
    };

    let saved_wallet = new_wallet.insert(&txn).await;
    if let Err(err) = saved_wallet {
        error!("Failed to save wallet for {}: {}", &claims.sub, err);
        let _ = txn.rollback().await;
        return HttpResponse::Found()
            .insert_header((http::header::LOCATION, "https://github.com/Greatchinex"))
            .finish();
    }

    let _ = txn.commit().await;

    HttpResponse::Found()
        .insert_header((
            http::header::LOCATION,
            "https://github.com/Greatchinex/money-transfer",
        ))
        .finish()
}

#[instrument(skip(body, req_user, app_state), fields(user_id = %req_user.uuid))]
pub async fn set_pin(
    body: web::Json<SetWithdrawalPinBody>,
    req_user: web::ReqData<users::Model>,
    app_state: web::Data<AppState>,
) -> impl Responder {
    let request_payload = match body.validate() {
        Ok(_) => body.into_inner(),
        Err(err) => {
            return HttpResponse::BadRequest()
                .json(json!({ "status": "error", "message": "Validation errors", "data": err }));
        }
    };

    let is_valid_password = validate_password(
        &req_user.password,
        &request_payload.password,
        &app_state.env.hash_key,
    );
    if !is_valid_password {
        return HttpResponse::BadRequest()
            .json(json!({ "status": "error",  "message": "Wrong password provided" }));
    }

    // If user is setting PIN for the first time or is changing an existing PIN
    let is_pin_reset = match &req_user.withdrawal_pin {
        Some(_) => true,
        None => false,
    };

    if is_pin_reset == true && request_payload.current_pin.is_none() {
        return HttpResponse::BadRequest()
            .json(json!({ "status": "error",  "message": "Please pass your current PIN" }));
    }

    if is_pin_reset == true {
        let hashed_pin = match &req_user.withdrawal_pin {
            Some(hashed_pin) => format!("{}", hashed_pin),
            None => String::new(),
        };

        let is_current_pin_valid = validate_password(
            &hashed_pin,
            &request_payload.current_pin.unwrap_or_default(),
            &app_state.env.hash_key,
        );
        if !is_current_pin_valid {
            return HttpResponse::BadRequest().json(
                json!({ "status": "error",  "message": "Current PIN you provided is incorrect" }),
            );
        }

        // Make sure new pin is not the same as old pin
        let check_new_pin = validate_password(
            &hashed_pin,
            &request_payload.new_pin,
            &app_state.env.hash_key,
        );
        if check_new_pin == true {
            return HttpResponse::BadRequest().json(
                json!({ "status": "error",  "message": "New PIN cannot be the same as old PIN" }),
            );
        }
    }

    let mut hasher = Hasher::default();
    let hashed_password = hasher
        .with_password(request_payload.new_pin)
        .with_secret_key(&app_state.env.hash_key)
        .hash();

    let hashed_password = match hashed_password {
        Ok(hashed_password) => hashed_password,
        Err(err) => {
            error!("Failed to hash PIN ===> {}", err);
            return HttpResponse::InternalServerError()
                .json(json!({ "status": "error", "message": "An unexpected error occured" }));
        }
    };

    let user_id = format!("{}", &req_user.uuid);
    let exec = app_state
        .db
        .execute(Statement::from_sql_and_values(
            DbBackend::MySql,
            r#"
                UPDATE users
                SET withdrawal_pin = ?, updated_at = ?
                WHERE uuid = ?
            ;"#,
            [hashed_password.into(), Utc::now().into(), user_id.into()],
        ))
        .await;

    match exec {
        Ok(_) => {
            return HttpResponse::Ok().json(json!({
                "status": "success",
                "message": "PIN set successfully"
            }));
        }
        Err(err) => {
            error!("DB error updating user PIN ===> {}", err);
            return HttpResponse::BadRequest().json(json!({
                "status": "error",  "message": "Error setting user PIN, Please try again later"
            }));
        }
    }
}
