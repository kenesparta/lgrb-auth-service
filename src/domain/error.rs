use crate::domain::UserError;

#[derive(Debug, thiserror::Error)]
pub enum AuthAPIError {
    #[error("User already exists")]
    UserAlreadyExists,

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Unexpected error")]
    UnexpectedError,

    #[error("Password error")]
    PasswordError(#[from] UserError),
}
