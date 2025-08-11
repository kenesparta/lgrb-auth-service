use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub struct LoginAttemptId(String);

#[derive(Debug, thiserror::Error)]
pub enum LoginAttemptIdError {
    #[error("Invalid UUID format")]
    InvalidFormat,
}

impl LoginAttemptId {
    pub fn new(id: String) -> Result<Self, LoginAttemptIdError> {
        match Uuid::parse_str(&id) {
            Ok(_) => Ok(LoginAttemptId(id)),
            Err(_) => Err(LoginAttemptIdError::InvalidFormat),
        }
    }
}

impl Default for LoginAttemptId {
    fn default() -> Self {
        Self(Uuid::new_v4().to_string())
    }
}

impl AsRef<str> for LoginAttemptId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
