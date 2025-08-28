use secrecy::{ExposeSecret, SecretBox};
use std::hash::{Hash, Hasher};

#[derive(Debug)]
pub struct Password(SecretBox<String>);

#[derive(Debug, thiserror::Error)]
pub enum PasswordError {
    #[error("Invalid email format")]
    InvalidFormat,
}

impl PartialEq for Password {
    fn eq(
        &self,
        other: &Self,
    ) -> bool {
        self.0.expose_secret() == other.0.expose_secret()
    }
}

impl Hash for Password {
    fn hash<H: Hasher>(
        &self,
        state: &mut H,
    ) {
        self.0.expose_secret().hash(state);
    }
}

impl Clone for Password {
    fn clone(&self) -> Self {
        let secret_value = self.0.expose_secret().clone();
        Password(SecretBox::new(Box::new(secret_value)))
    }
}

impl Password {
    pub fn new(password: SecretBox<String>) -> Result<Self, PasswordError> {
        let password_object = Password(password);
        if !password_object.is_correct_password() {
            return Err(PasswordError::InvalidFormat);
        }

        Ok(password_object)
    }

    pub fn is_correct_password(&self) -> bool {
        let pass_size = self.0.expose_secret().len();
        pass_size >= 8 && pass_size <= 128
    }
}

impl AsRef<SecretBox<String>> for Password {
    fn as_ref(&self) -> &SecretBox<String> {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::Password;
    use fake::Fake;
    use fake::faker::internet::en::Password as FakePassword;
    use secrecy::{ExposeSecret, SecretBox};

    #[test]
    fn test_valid_password_creation() {
        let valid_password = "12345678";
        let secret = SecretBox::new(Box::new(valid_password.to_string()));
        let password = Password::new(secret);
        assert!(password.is_ok());
    }

    #[test]
    fn test_valid_password_with_maximum_length() {
        let valid_password = "a".repeat(128);
        let secret = SecretBox::new(Box::new(valid_password));
        let password = Password::new(secret);
        assert!(password.is_ok());
    }

    #[test]
    fn test_password_too_short() {
        // Test with password shorter than 8 characters
        let short_password = "1234567"; // 7 characters
        let secret = SecretBox::new(Box::new(short_password.to_string()));
        let password = Password::new(secret);
        assert!(password.is_err());
        assert!(matches!(password.unwrap_err(), super::PasswordError::InvalidFormat));
    }

    #[test]
    fn test_password_too_long() {
        // Test with password longer than 128 characters
        let long_password = "a".repeat(129);
        let secret = SecretBox::new(Box::new(long_password));
        let password = Password::new(secret);
        assert!(password.is_err());
        assert!(matches!(password.unwrap_err(), super::PasswordError::InvalidFormat));
    }

    #[test]
    fn test_empty_password() {
        let empty_password = "";
        let secret = SecretBox::new(Box::new(empty_password.to_string()));
        let password = Password::new(secret);
        assert!(password.is_err());
        assert!(matches!(password.unwrap_err(), super::PasswordError::InvalidFormat));
    }

    #[test]
    fn test_password_equality() {
        let password_str = "validpassword123";
        let secret1 = SecretBox::new(Box::new(password_str.to_string()));
        let secret2 = SecretBox::new(Box::new(password_str.to_string()));

        let password1 = Password::new(secret1).unwrap();
        let password2 = Password::new(secret2).unwrap();

        assert_eq!(password1, password2);
    }

    #[test]
    fn test_password_inequality() {
        let password_str1 = "validpassword123";
        let password_str2 = "differentpassword456";
        let secret1 = SecretBox::new(Box::new(password_str1.to_string()));
        let secret2 = SecretBox::new(Box::new(password_str2.to_string()));

        let password1 = Password::new(secret1).unwrap();
        let password2 = Password::new(secret2).unwrap();

        assert_ne!(password1, password2);
    }

    #[test]
    fn test_is_correct_password() {
        // Test valid password
        let valid_password = "validpass123";
        let secret = SecretBox::new(Box::new(valid_password.to_string()));
        let password = Password::new(secret).unwrap();
        assert!(password.is_correct_password());

        // Test invalid password (too short)
        let invalid_secret = SecretBox::new(Box::new("short".to_string()));
        let invalid_password = Password(invalid_secret);
        assert!(!invalid_password.is_correct_password());
    }

    #[test]
    fn test_as_ref() {
        let password_str = "testpassword123";
        let secret = SecretBox::new(Box::new(password_str.to_string()));
        let password = Password::new(secret).unwrap();

        let secret_ref: &SecretBox<String> = password.as_ref();
        assert_eq!(secret_ref.expose_secret(), password_str);
    }

    #[test]
    fn test_fake_password_generation() {
        // Test with fake password generation
        let fake_password: String = FakePassword(8..20).fake();
        let secret = SecretBox::new(Box::new(fake_password));
        let password = Password::new(secret);
        assert!(password.is_ok());
    }

    #[test]
    fn test_boundary_conditions() {
        // Test exact boundary conditions
        let min_password = "a".repeat(8); // Exactly 8 characters
        let max_password = "a".repeat(128); // Exactly 128 characters

        let min_secret = SecretBox::new(Box::new(min_password));
        let max_secret = SecretBox::new(Box::new(max_password));

        assert!(Password::new(min_secret).is_ok());
        assert!(Password::new(max_secret).is_ok());
    }
}
