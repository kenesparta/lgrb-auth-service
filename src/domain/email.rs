use validator::ValidateEmail;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Email(String);

#[derive(Debug, thiserror::Error)]
pub enum EmailError {
    #[error("Invalid email format")]
    InvalidFormat,
}

impl Email {
    pub fn new(email: String) -> Result<Self, EmailError> {
        if !ValidateEmail::validate_email(&email) {
            return Err(EmailError::InvalidFormat);
        }

        Ok(Self(email.to_string()))
    }
}

impl AsRef<str> for Email {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}
