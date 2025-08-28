use crate::domain::Email;
use color_eyre::Report;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EmailClientError {
    #[error("Unexpected error")]
    UnexpectedError(#[source] Report),
}

impl PartialEq for EmailClientError {
    fn eq(
        &self,
        other: &Self,
    ) -> bool {
        matches!((self, other), (Self::UnexpectedError(_), Self::UnexpectedError(_)))
    }
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
