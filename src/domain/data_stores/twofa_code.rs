use crate::domain::Email;
use crate::domain::loggin_attempt::LoginAttemptId;
use crate::domain::twofa_code::TwoFACode;

#[cfg(test)]
use mockall::automock;

#[derive(Debug, PartialEq)]
pub enum TwoFACodeStoreError {
    LoginAttemptIdNotFound,
    UnexpectedError,
    UserNotFound,
}

// This trait represents the interface all concrete 2FA code stores should implement
#[cfg_attr(test, automock)]
#[async_trait::async_trait]
pub trait TwoFACodeStore: Send + Sync {
    async fn add_code(
        &mut self,
        email: Email,
        login_attempt_id: LoginAttemptId,
        code: TwoFACode,
    ) -> Result<(), TwoFACodeStoreError>;
    async fn remove_code(&mut self, email: &Email) -> Result<(), TwoFACodeStoreError>;
    async fn get_code<'two_fa>(
        &'two_fa self,
        email: &Email,
    ) -> Result<&'two_fa (LoginAttemptId, TwoFACode), TwoFACodeStoreError>;
}
