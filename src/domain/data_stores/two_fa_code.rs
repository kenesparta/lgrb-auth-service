use crate::domain::Email;
use crate::domain::login_attempt::LoginAttemptId;
use crate::domain::two_fa_code::TwoFACode;
use color_eyre::Report;
use thiserror::Error;

#[cfg(test)]
use mockall::automock;

#[derive(Debug, Error)]
pub enum TwoFACodeStoreError {
    #[error("Login Attempt ID not found")]
    LoginAttemptIdNotFound,

    #[error("Unexpected error")]
    UnexpectedError(#[source] Report),

    #[error("User not found")]
    UserNotFound,
}

impl PartialEq for TwoFACodeStoreError {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (Self::LoginAttemptIdNotFound, Self::LoginAttemptIdNotFound)
                | (Self::UserNotFound, Self::UserNotFound)
                | (Self::UnexpectedError(_), Self::UnexpectedError(_))
        )
    }
}

// This trait represents the interface all concrete 2FA code stores should implement
#[cfg_attr(test, automock)]
#[async_trait::async_trait]
pub trait TwoFACodeStore: Send + Sync {
    async fn add_code(
        &mut self,
        email: &Email,
        login_attempt_id: LoginAttemptId,
        code: TwoFACode,
    ) -> Result<(), TwoFACodeStoreError>;
    async fn remove_code(&mut self, email: &Email) -> Result<(), TwoFACodeStoreError>;
    async fn get_code(
        &self,
        email: &Email,
    ) -> Result<(LoginAttemptId, TwoFACode), TwoFACodeStoreError>;
}
