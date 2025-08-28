use color_eyre::eyre::{Report, Result};
use rand::Rng;
use secrecy::{ExposeSecret, SecretBox};
use std::fmt;

#[derive(Debug)]
pub struct TwoFACode(SecretBox<String>);

#[derive(Debug, thiserror::Error)]
pub enum TwoFACodeError {
    #[error("2FA code must be exactly 6 characters long")]
    InvalidLength,

    #[error("2FA code must contain only digits")]
    InvalidFormat,
}

impl TwoFACode {
    pub fn new(code: SecretBox<String>) -> Result<Self> {
        if code.expose_secret().len() != 6 {
            return Err(Report::from(TwoFACodeError::InvalidLength));
        }

        if !code.expose_secret().chars().all(|c| c.is_ascii_digit()) {
            return Err(Report::from(TwoFACodeError::InvalidFormat));
        }

        Ok(TwoFACode(code))
    }

    pub fn code(self) -> SecretBox<String> {
        self.0
    }
}

impl Default for TwoFACode {
    fn default() -> Self {
        let mut rng = rand::rng();
        let code = (0..6).map(|_| rng.random_range(0..10).to_string()).collect::<String>();

        TwoFACode(SecretBox::new(Box::from(code)))
    }
}

impl PartialEq for TwoFACode {
    fn eq(
        &self,
        other: &Self,
    ) -> bool {
        self.0.expose_secret() == other.0.expose_secret()
    }
}

impl Clone for TwoFACode {
    fn clone(&self) -> Self {
        Self(SecretBox::new(Box::from(self.0.expose_secret().clone())))
    }
}

impl AsRef<SecretBox<String>> for TwoFACode {
    fn as_ref(&self) -> &SecretBox<String> {
        &self.0
    }
}

impl fmt::Display for TwoFACode {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        write!(f, "{}", self.0.expose_secret())
    }
}
