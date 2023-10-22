use async_trait::async_trait;
use lettre::{
    message::header::ContentType, transport::smtp::authentication::Credentials, AsyncSmtpTransport,
    AsyncTransport, Message, Tokio1Executor,
};
use tracing::{error, info, instrument};

use super::config::EnvConfig;

pub struct SendEmail {
    pub to: String,
    pub from: String,
    pub subject: String,
    pub template: String,
}

#[async_trait]
pub trait SendEmailTrait {
    async fn send_email(&self, env: &EnvConfig) -> Result<(), ()>;
}

#[async_trait]
impl SendEmailTrait for SendEmail {
    #[instrument(skip(self, env), fields(email = %self.to, subject = %self.subject))]
    async fn send_email(&self, env: &EnvConfig) -> Result<(), ()> {
        let email = Message::builder()
            .to(self.to.parse().unwrap())
            .from(self.from.parse().unwrap())
            .subject(format!("{}", self.subject))
            .header(ContentType::TEXT_HTML)
            .body(format!("{}", self.template))
            .expect("Failed to build email");

        let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(&env.smtp_provider)
            .unwrap()
            .credentials(Credentials::new(
                format!("{}", &env.smtp_user),
                format!("{}", &env.smtp_key),
            ))
            .build();

        match mailer.send(email).await {
            Ok(_) => info!("Email successfully sent"),
            Err(err) => error!("Failed to deliver email: {}", err),
        }

        Ok(())
    }
}
