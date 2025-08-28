use color_eyre::eyre::{Context, Result};
use secrecy::{ExposeSecret, SecretBox};
use uuid::Uuid;

#[derive(Debug)]
pub struct LoginAttemptId(SecretBox<String>);

#[derive(Debug, thiserror::Error)]
pub enum LoginAttemptIdError {
    #[error("Invalid UUID format")]
    InvalidFormat,
}

impl LoginAttemptId {
    pub fn new(id: SecretBox<String>) -> Result<Self> {
        let parsed_id = Uuid::parse_str(&id.expose_secret()).wrap_err("Invalid login attempt id")?;
        Ok(LoginAttemptId(SecretBox::new(Box::from(parsed_id.to_string()))))
    }

    pub fn id(self) -> SecretBox<String> {
        self.0
    }
}

impl PartialEq for LoginAttemptId {
    fn eq(
        &self,
        other: &Self,
    ) -> bool {
        self.0.expose_secret().eq(other.0.expose_secret())
    }
}

impl Clone for LoginAttemptId {
    fn clone(&self) -> Self {
        Self(SecretBox::new(Box::new(self.0.expose_secret().clone())))
    }
}

impl Default for LoginAttemptId {
    fn default() -> Self {
        Self(SecretBox::new(Box::from(Uuid::new_v4().to_string())))
    }
}

impl AsRef<SecretBox<String>> for LoginAttemptId {
    fn as_ref(&self) -> &SecretBox<String> {
        &self.0
    }
}
