use rand::Rng;

#[derive(Clone, Debug, PartialEq)]
pub struct TwoFACode(String);

#[derive(Debug, thiserror::Error)]
pub enum TwoFACodeError {
    #[error("2FA code must be exactly 6 characters long")]
    InvalidLength,

    #[error("2FA code must contain only digits")]
    InvalidFormat,
}

impl TwoFACode {
    pub fn new(code: String) -> Result<Self, TwoFACodeError> {
        if code.len() != 6 {
            return Err(TwoFACodeError::InvalidLength);
        }

        if !code.chars().all(|c| c.is_ascii_digit()) {
            return Err(TwoFACodeError::InvalidFormat);
        }

        Ok(TwoFACode(code))
    }
}

impl Default for TwoFACode {
    fn default() -> Self {
        let mut rng = rand::rng();
        let code = (0..6)
            .map(|_| rng.random_range(0..10).to_string())
            .collect::<String>();

        TwoFACode(code)
    }
}

impl AsRef<str> for TwoFACode {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
