use crate::domain::Email;

#[derive(Debug, PartialEq)]
pub enum EmailClientError {
    UnexpectedError,
}

// This trait represents the interface all concrete email clients should implement
#[async_trait::async_trait]
pub trait EmailClient: Send + Sync {
    async fn send_email(
        &self,
        recipient: &Email,
        subject: &str,
        content: &str,
    ) -> Result<(), EmailClientError>;
}
