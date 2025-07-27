#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Password(String);

#[derive(Debug, thiserror::Error)]
pub enum PasswordError {
    #[error("Invalid email format")]
    InvalidFormat,
}

impl Password {
    pub fn new(password: String) -> Result<Self, PasswordError> {
        let password_object = Password(password);
        if !password_object.is_correct_password() {
            return Err(PasswordError::InvalidFormat);
        }

        Ok(password_object)
    }

    pub fn is_correct_password(&self) -> bool {
        let pass_size = self.0.len();
        pass_size >= 8 && pass_size <= 128
    }
}

impl AsRef<str> for Password {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
