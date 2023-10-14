use lettre::{
    message::header::ContentType, transport::smtp::authentication::Credentials, AsyncSmtpTransport,
    AsyncTransport, Message, Tokio1Executor,
};
use std::env;
use tracing::{error, info, instrument};

pub struct SendEmail {
    pub to: String,
    pub from: String,
    pub subject: String,
    pub template: String,
}

trait SendEmailTrait {
    pub async fn send_email(&self) -> Result<()>;
}

impl SendEmailTrait for SendEmail {
    #[tokio::main]
    #[instrument(skip(self), fields(user_email = %self.to, subject = %self.subject))]
    async fn send_email(&self) -> Result<()> {
        let email = Message::builder()
            .to(self.to.parse().unwrap())
            .from(self.from.parse().unwrap())
            .subject(self.subject)
            .header(ContentType::TEXT_HTML)
            .body(String::from(self.template))
            .expect("Failed to build email");

        let smtp_provider =
            env::var("SMTP_PROVIDER").expect("SMTP_PROVIDER is not set in .env file");
        let smtp_user = env::var("SMTP_USER").expect("SMTP_USER is not set in .env file");
        let smtp_key = env::var("SMTP_KEY").expect("SMTP_KEY is not set in .env file");

        let mailer: AsyncSmtpTransport<Tokio1Executor> =
            AsyncSmtpTransport::<Tokio1Executor>::relay(smtp_provider)
                .unwrap()
                .credentials(Credentials::new(smtp_user, smtp_key))
                .build();

        match mailer.send(email).await {
            Ok(_) => info!("Email successfully sent"),
            Err(err) => error!("Failed to deliver email: {}", err),
        }

        Ok(())
    }
}
