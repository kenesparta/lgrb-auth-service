use crate::domain::{Email, EmailError, Password, PasswordError};
use secrecy::SecretBox;

#[derive(Debug, Clone, PartialEq)]
pub struct User {
    email: Email,
    password: Password,
    requires_2fa: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum UserError {
    #[error("Invalid email format")]
    InvalidFormat,

    #[error("Email Error")]
    EmailError(#[from] EmailError),

    #[error("Password error")]
    PasswordError(#[from] PasswordError),
}

impl User {
    pub fn new(
        email: String,
        password: String,
        requires_2fa: bool,
    ) -> Result<Self, UserError> {
        let email = Email::new(SecretBox::new(Box::from(email)))?;
        let password = Password::new(SecretBox::new(Box::from(password)))?;
        Ok(Self {
            email,
            password,
            requires_2fa,
        })
    }

    pub fn email(&self) -> &Email {
        &self.email
    }

    pub fn password(&self) -> &Password {
        &self.password
    }

    pub fn requires_2fa(&self) -> bool {
        self.requires_2fa
    }
}
