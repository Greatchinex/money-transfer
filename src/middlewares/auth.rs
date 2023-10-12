use actix_web::{
    dev::Payload, error::ErrorUnauthorized, http, Error as ActixWebError, FromRequest, HttpMessage,
    HttpRequest,
};
use futures::future::{err, ok, Ready};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde_json::json;
use std::env;
use tracing::error;
use uuid::Uuid;

use crate::dto::users::TokenClaims;

pub struct AuthMiddleware {
    pub user_id: Uuid,
}

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
