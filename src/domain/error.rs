use crate::domain::{EmailError, PasswordError, UserError};
use crate::utils::GenerateTokenError;
use color_eyre::Report;

#[derive(Debug, thiserror::Error)]
pub enum AuthAPIError {
    #[error("User already exists")]
    UserAlreadyExists,

    #[error("Incorrect credentials")]
    IncorrectCredentials,

    #[error("Email or password incorrect")]
    EmailOrPasswordIncorrect,

    #[error("Unexpected error")]
    UnexpectedError(#[source] Report),

    #[error("Error adding to banned tokens")]
    ErrorAddingToBannedTokens,

    #[error("Missing token")]
    MissingToken,

    #[error("Token not valid")]
    TokenNotValid,

    #[error("Password error")]
    PasswordError(#[from] PasswordError),

    #[error("Email error")]
    EmailError(#[from] EmailError),

    #[error("User error")]
    UserError(#[from] UserError),

    #[error("Generate token error")]
    GenerateTokenError(#[from] GenerateTokenError),

    #[error("TwoFA malformed error")]
    TwoFAMalformedError,

    #[error("Login attempt id malformed error")]
    LoginAttemptIdMalformedError,
}
