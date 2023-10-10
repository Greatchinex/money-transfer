use serde::Deserialize;
use validator::Validate;

#[derive(Deserialize, Validate, Debug)]
pub struct SignupBody {
    #[validate(length(min = 1, max = 255))]
    pub first_name: String,

    #[validate(length(min = 1, max = 255))]
    pub last_name: String,

    #[validate(email(message = "Email must be a valid email type"))]
    pub email: String,

    #[validate(length(min = 1, message = "Password must not be empty"))]
    pub password: String,
}
