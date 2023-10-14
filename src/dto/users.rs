use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Deserialize, Validate, Debug)]
pub struct SignupBody {
    #[validate(length(min = 1, max = 255))]
    pub first_name: String,

    #[validate(length(min = 1, max = 255))]
    pub last_name: String,

    #[validate(email(message = "Email must be a valid email type"))]
    pub email: String,

    #[validate(length(min = 3, message = "Password must be minimum of three(3) characters"))]
    pub password: String,
}

#[derive(Deserialize, Validate, Debug)]
pub struct LoginBody {
    #[validate(email(message = "Email must be a valid email type"))]
    pub email: String,

    #[validate(length(min = 3, message = "Password must be minimum of three(3) characters"))]
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: String,
    pub auth_type: String,
    pub exp: usize,
    pub iat: usize,
}
