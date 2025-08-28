use secrecy::{ExposeSecret, SecretBox};
use std::hash::{Hash, Hasher};
use validator::ValidateEmail;

#[derive(Debug)]
pub struct Email(SecretBox<String>);

#[derive(Debug, thiserror::Error)]
pub enum EmailError {
    #[error("Invalid email format")]
    InvalidFormat,
}

impl Email {
    pub fn new(email: SecretBox<String>) -> Result<Self, EmailError> {
        if !ValidateEmail::validate_email(&email.expose_secret()) {
            return Err(EmailError::InvalidFormat);
        }

        Ok(Self(email))
    }
}

impl Clone for Email {
    fn clone(&self) -> Self {
        Self(SecretBox::new(Box::new(self.0.expose_secret().clone())))
    }
}

impl PartialEq for Email {
    fn eq(
        &self,
        other: &Self,
    ) -> bool {
        self.0.expose_secret() == other.0.expose_secret()
    }
}

impl Eq for Email {}

impl Hash for Email {
    fn hash<H: Hasher>(
        &self,
        state: &mut H,
    ) {
        self.0.expose_secret().hash(state);
    }
}

impl AsRef<SecretBox<String>> for Email {
    fn as_ref(&self) -> &SecretBox<String> {
        &self.0
    }
}
