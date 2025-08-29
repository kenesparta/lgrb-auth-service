use crate::domain::Email;
use crate::domain::client::{EmailClient, EmailClientError};
use secrecy::ExposeSecret;

pub struct MockEmailClient;

impl MockEmailClient {
    pub fn new() -> Self {
        MockEmailClient
    }
}

#[async_trait::async_trait]
impl EmailClient for MockEmailClient {
    async fn send_email(
        &self,
        recipient: &Email,
        subject: &str,
        content: &str,
    ) -> Result<(), EmailClientError> {
        println!(
            "Sending email to {} with subject: {} and content: {}",
            recipient.as_ref().expose_secret(),
            subject,
            content
        );

        Ok(())
    }
}
