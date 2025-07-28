use crate::domain::{EmailError, PasswordError, UserError};

#[derive(Debug, thiserror::Error)]
pub enum AuthAPIError {
    #[error("User already exists")]
    UserAlreadyExists,

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Unexpected error")]
    UnexpectedError,

    #[error("Password error")]
    PasswordError(#[from] PasswordError),

    #[error("Email error")]
    EmailError(#[from] EmailError),

    #[error("User error")]
    UserError(#[from] UserError),
}
