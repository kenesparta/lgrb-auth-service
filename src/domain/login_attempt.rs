use color_eyre::eyre::{Context, Result};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub struct LoginAttemptId(String);

#[derive(Debug, thiserror::Error)]
pub enum LoginAttemptIdError {
    #[error("Invalid UUID format")]
    InvalidFormat,
}

impl LoginAttemptId {
    pub fn new(id: String) -> Result<Self> {
        let parsed_id = Uuid::parse_str(&id).wrap_err("Invalid login attempt id")?;
        Ok(LoginAttemptId(parsed_id.to_string()))
    }

    pub fn id(self) -> String {
        self.0
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
