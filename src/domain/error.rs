use crate::domain::{EmailError, PasswordError, UserError};

#[derive(Debug, thiserror::Error)]
pub enum AuthAPIError {
    #[error("User already exists")]
    UserAlreadyExists,

    #[error("Incorrect credentials")]
    IncorrectCredentials,

    #[error("Email or password incorrect")]
    EmailOrPasswordIncorrect,

    #[error("Unexpected error")]
    UnexpectedError,

    #[error("Password error")]
    PasswordError(#[from] PasswordError),

    #[error("Email error")]
    EmailError(#[from] EmailError),

    #[error("User error")]
    UserError(#[from] UserError),
}
