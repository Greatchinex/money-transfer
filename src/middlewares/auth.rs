use actix_web::{
    body::MessageBody,
    dev::{Payload, ServiceRequest, ServiceResponse},
    error::ErrorUnauthorized,
    http, web, Error as ActixWebError, FromRequest, HttpMessage, HttpRequest,
};
use actix_web_lab::middleware::Next;
use futures::future::{err, ok, Ready};
use jsonwebtoken::{decode, DecodingKey, Validation};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde_json::json;
use std::env;
use tracing::error;
use uuid::Uuid;

use crate::dto::users::TokenClaims;
use crate::entities::{prelude::Users, users::Column};
use crate::AppState;

pub async fn auth_middleware(
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, ActixWebError> {
    let authorization = req.headers().get(http::header::AUTHORIZATION);
    let authorization = match authorization {
        Some(auth) => auth,
        None => {
            return Err(ErrorUnauthorized(
                json!({ "status": "error", "message": "No auth header found" }),
            ))
        }
    };

    let auth_parts: Vec<&str> = authorization.to_str().unwrap().split_whitespace().collect();
    if auth_parts.len() != 2 || auth_parts[0] != "Bearer" {
        return Err(ErrorUnauthorized(
            json!({ "status": "error", "message": "Improper auth header format" }),
        ));
    }

    let token = auth_parts[1].trim();
    let token_secret = env::var("APP_KEY").expect("APP_KEY is not set in .env file");
    let claims = match decode::<TokenClaims>(
        token,
        &DecodingKey::from_secret(token_secret.as_ref()),
        &Validation::default(),
    ) {
        Ok(c) => c.claims,
        Err(err) => {
            error!("AuthMiddleware: error decoding token ===> {}", err);
            return Err(ErrorUnauthorized(
                json!({ "status": "error", "message": "Please login again" }),
            ));
        }
    };

    if claims.auth_type != "USER_AUTH" {
        return Err(ErrorUnauthorized(
            json!({ "status": "error", "message": "Invalid auth" }),
        ));
    }

    let app_state = req.app_data::<web::Data<AppState>>().unwrap();

    let user = Users::find()
        .filter(Column::Uuid.eq(claims.sub))
        .one(&app_state.db)
        .await;

    let user = match user {
        Ok(Some(user)) => user,
        Ok(None) => {
            return Err(ErrorUnauthorized(
                json!({ "status": "error", "message": "Invalid user" }),
            ));
        }
        Err(err) => {
            error!("AuthMiddleware: DB error validating user ===> {}", err);
            return Err(ErrorUnauthorized(
                json!({ "status": "error", "message": "An unexpected error occured" }),
            ));
        }
    };

    req.extensions_mut().insert(user);

    next.call(req).await
}

pub struct AuthMiddleware {
    pub user_id: Uuid,
}

// NOTE: Older implementation below swapped out for a simpler "auth_middleware" method above. Dont want to remove below implementation
// Incase i need to reference it in the future
impl FromRequest for AuthMiddleware {
    type Error = ActixWebError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let authorization = req.headers().get(http::header::AUTHORIZATION);
        let authorization = match authorization {
            Some(auth) => auth,
            None => {
                return err(ErrorUnauthorized(
                    json!({ "status": "error", "message": "No auth header found" }),
                ))
            }
        };

        let auth_parts: Vec<&str> = authorization.to_str().unwrap().split_whitespace().collect();
        if auth_parts.len() != 2 || auth_parts[0] != "Bearer" {
            return err(ErrorUnauthorized(
                json!({ "status": "error", "message": "Improper auth header format" }),
            ));
        }

        let token = auth_parts[1].trim();
        let token_secret = env::var("APP_KEY").expect("APP_KEY is not set in .env file");
        let claims = match decode::<TokenClaims>(
            token,
            &DecodingKey::from_secret(token_secret.as_ref()),
            &Validation::default(),
        ) {
            Ok(c) => c.claims,
            Err(error) => {
                error!("AuthMiddleware error decoding token ===> {}", error);
                return err(ErrorUnauthorized(
                    json!({ "status": "error", "message": "Please login again" }),
                ));
            }
        };

        let user_id = Uuid::parse_str(claims.sub.as_str()).unwrap();
        req.extensions_mut().insert(user_id.to_owned());

        ok(AuthMiddleware { user_id })
    }
}
