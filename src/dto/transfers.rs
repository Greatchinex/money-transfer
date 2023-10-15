use serde::Deserialize;
use validator::Validate;

#[derive(Deserialize, Validate, Debug)]
pub struct InitiateFundingBody {
    #[validate(range(min = 100, message = "Minimum funding amount is 100 Naira"))]
    pub amount: u64, // Amount in Naira

    #[validate(length(min = 3, message = "Password must be minimum of three(3) characters"))]
    pub password: String,
}
