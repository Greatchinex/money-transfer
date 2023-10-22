use serde::Deserialize;
use validator::Validate;

#[derive(Deserialize, Validate, Debug)]
pub struct InitiateFundingBody {
    #[validate(range(min = 100, message = "Minimum funding amount is 100 Naira"))]
    pub amount: u64, // Amount in Naira

    #[validate(length(min = 3, message = "Password must be minimum of three(3) characters"))]
    pub password: String,
}

#[derive(Deserialize, Validate, Debug)]
pub struct P2PTransferBody {
    #[validate(range(min = 100, message = "Minimum transfer amount is 100 Naira"))]
    pub amount: u64,

    #[validate(length(min = 6, max = 6, message = "PIN must be Six(6) characters long"))]
    pub pin: String,

    #[validate(length(min = 4))]
    pub receiver_id: String,

    #[validate(length(min = 4, max = 255))]
    pub narration: Option<String>,
}
