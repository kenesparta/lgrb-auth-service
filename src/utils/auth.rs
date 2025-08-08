use super::constants::JWT_COOKIE_NAME;
use crate::domain::Email;
use crate::utils::jwt::{COOKIE_DOMAIN, JWT_SECRET};
use crate::utils::{JWT_REFRESH_COOKIE_NAME, REFRESH_TOKEN_TTL_SECONDS, TOKEN_TTL_SECONDS};
use axum_extra::extract::cookie::Cookie;
use chrono::Utc;
use jsonwebtoken::{DecodingKey, EncodingKey, Validation, decode, encode};
use serde::{Deserialize, Serialize};

// Create a cookie with a new JWT auth token
pub fn generate_auth_cookie(email: &Email) -> Result<Cookie<'static>, GenerateTokenError> {
    let token = generate_token_pair(email)?;
    let cookie = Cookie::build((JWT_COOKIE_NAME, token.access_token))
        .domain(COOKIE_DOMAIN.as_str())
        .path("/")
        .http_only(true)
        .build();
    Ok(cookie)
}

pub fn generate_refresh_cookie(email: &Email) -> Result<Cookie<'static>, GenerateTokenError> {
    let token = generate_token_pair(email)?;
    let cookie = Cookie::build((JWT_REFRESH_COOKIE_NAME, token.refresh_token))
        .domain(COOKIE_DOMAIN.as_str())
        .path("/")
        .http_only(true)
        .build();

    Ok(cookie)
}

#[derive(Debug, thiserror::Error)]
pub enum GenerateTokenError {
    #[error("Error creating JWT token")]
    TokenError(jsonwebtoken::errors::Error),

    #[error("Unexpected error generating JWT token")]
    UnexpectedError,
}

// Check if the JWT auth token is valid by decoding it using the JWT secret
pub async fn validate_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(JWT_SECRET.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
}

// Create a JWT auth token by encoding claims using the JWT secret
fn create_token(claims: &Claims) -> Result<String, jsonwebtoken::errors::Error> {
    encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET.as_bytes()),
    )
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub token_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
}

pub fn generate_token_pair(email: &Email) -> Result<TokenPair, GenerateTokenError> {
    let delta = chrono::Duration::try_seconds(TOKEN_TTL_SECONDS)
        .ok_or(GenerateTokenError::UnexpectedError)?;
    let access_exp = Utc::now()
        .checked_add_signed(delta)
        .ok_or(GenerateTokenError::UnexpectedError)?
        .timestamp();

    let delta = chrono::Duration::try_seconds(REFRESH_TOKEN_TTL_SECONDS)
        .ok_or(GenerateTokenError::UnexpectedError)?;
    let refresh_exp = Utc::now()
        .checked_add_signed(delta)
        .ok_or(GenerateTokenError::UnexpectedError)?
        .timestamp();

    let access_claims = Claims {
        sub: email.as_ref().to_string(),
        exp: access_exp
            .try_into()
            .map_err(|_| GenerateTokenError::UnexpectedError)?,
        token_type: "access".to_string(),
    };

    let refresh_claims = Claims {
        sub: email.as_ref().to_string(),
        exp: refresh_exp
            .try_into()
            .map_err(|_| GenerateTokenError::UnexpectedError)?,
        token_type: "refresh".to_string(),
    };

    Ok(TokenPair {
        access_token: create_token(&access_claims).map_err(GenerateTokenError::TokenError)?,
        refresh_token: create_token(&refresh_claims).map_err(GenerateTokenError::TokenError)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use fake::Fake;
    use fake::faker::internet::en::SafeEmail;

    #[tokio::test]
    async fn test_generate_auth_cookie() {
        let fake_email: String = SafeEmail().fake();
        let email = Email::new(fake_email.to_owned()).unwrap();
        let cookie = generate_auth_cookie(&email).unwrap();
        assert_eq!(cookie.name(), JWT_COOKIE_NAME);
        assert_eq!(cookie.value().split('.').count(), 3);
        assert_eq!(cookie.path(), Some("/"));
        assert_eq!(cookie.http_only(), Some(true));
    }

    #[tokio::test]
    async fn test_validate_token_with_invalid_token() {
        let token = "invalid_token".to_owned();
        let result = validate_token(&token).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_generate_token_pair_success() {
        let fake_email: String = SafeEmail().fake();
        let email = Email::new(fake_email.clone()).unwrap();

        let result = generate_token_pair(&email);

        assert!(result.is_ok());
        let token_pair = result.unwrap();

        assert!(!token_pair.access_token.is_empty());
        assert!(!token_pair.refresh_token.is_empty());
        assert_ne!(token_pair.access_token, token_pair.refresh_token);
        assert_eq!(token_pair.access_token.split('.').count(), 3);
        assert_eq!(token_pair.refresh_token.split('.').count(), 3);
    }

    #[tokio::test]
    async fn test_generate_token_pair_tokens_are_valid() {
        let fake_email: String = SafeEmail().fake();
        let email = Email::new(fake_email.clone()).unwrap();

        let token_pair = generate_token_pair(&email).unwrap();

        let access_claims = validate_token(&token_pair.access_token).await.unwrap();
        assert_eq!(access_claims.sub, fake_email);
        assert_eq!(access_claims.token_type, "access");

        let refresh_claims = validate_token(&token_pair.refresh_token).await.unwrap();
        assert_eq!(refresh_claims.sub, fake_email);
        assert_eq!(refresh_claims.token_type, "refresh");
    }

    #[tokio::test]
    async fn test_generate_token_pair_expiration_times() {
        let fake_email: String = SafeEmail().fake();
        let email = Email::new(fake_email).unwrap();

        let before_generation = Utc::now();
        let token_pair = generate_token_pair(&email).unwrap();
        let after_generation = Utc::now();

        let access_claims = validate_token(&token_pair.access_token).await.unwrap();
        let refresh_claims = validate_token(&token_pair.refresh_token).await.unwrap();

        let delta = chrono::Duration::try_seconds(TOKEN_TTL_SECONDS).unwrap();
        let expected_access_min =
            before_generation + delta - chrono::Duration::try_seconds(1).unwrap();
        let expected_access_max =
            after_generation + delta + chrono::Duration::try_seconds(1).unwrap();
        let access_exp_time =
            chrono::DateTime::from_timestamp(access_claims.exp as i64, 0).unwrap();

        assert!(access_exp_time >= expected_access_min);
        assert!(access_exp_time <= expected_access_max);

        let delta = chrono::Duration::try_seconds(REFRESH_TOKEN_TTL_SECONDS).unwrap();
        let expected_refresh_min =
            before_generation + delta - chrono::Duration::try_seconds(1).unwrap();
        let expected_refresh_max =
            after_generation + delta + chrono::Duration::try_seconds(1).unwrap();
        let refresh_exp_time =
            chrono::DateTime::from_timestamp(refresh_claims.exp as i64, 0).unwrap();

        assert!(refresh_exp_time >= expected_refresh_min);
        assert!(refresh_exp_time <= expected_refresh_max);

        if REFRESH_TOKEN_TTL_SECONDS > TOKEN_TTL_SECONDS {
            assert!(refresh_exp_time > access_exp_time);
        }
    }

    #[tokio::test]
    async fn test_generate_token_pair_different_emails_produce_different_tokens() {
        let email1 = Email::new("user1@example.com".to_string()).unwrap();
        let email2 = Email::new("user2@example.com".to_string()).unwrap();

        let token_pair1 = generate_token_pair(&email1).unwrap();
        let token_pair2 = generate_token_pair(&email2).unwrap();

        assert_ne!(token_pair1.access_token, token_pair2.access_token);
        assert_ne!(token_pair1.refresh_token, token_pair2.refresh_token);

        let claims1 = validate_token(&token_pair1.access_token).await.unwrap();
        let claims2 = validate_token(&token_pair2.access_token).await.unwrap();

        assert_eq!(claims1.sub, "user1@example.com");
        assert_eq!(claims2.sub, "user2@example.com");
    }

    #[tokio::test]
    async fn test_generate_token_pair_multiple_calls_produce_different_tokens() {
        let fake_email: String = SafeEmail().fake();
        let email = Email::new(fake_email).unwrap();

        let token_pair1 = generate_token_pair(&email).unwrap();
        tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;
        let token_pair2 = generate_token_pair(&email).unwrap();

        assert_ne!(token_pair1.access_token, token_pair2.access_token);
        assert_ne!(token_pair1.refresh_token, token_pair2.refresh_token);
    }

    #[tokio::test]
    async fn test_generate_token_pair_with_various_email_formats() {
        let fake_email1: String = SafeEmail().fake();
        let fake_email2: String = SafeEmail().fake();
        let fake_email3: String = SafeEmail().fake();
        let fake_email4: String = SafeEmail().fake();
        let fake_email5: String = SafeEmail().fake();
        let test_emails = vec![
            fake_email1,
            fake_email2,
            fake_email3,
            fake_email4,
            fake_email5,
        ];

        for email_str in test_emails {
            let email = Email::new(email_str.to_string()).unwrap();
            let result = generate_token_pair(&email);

            assert!(
                result.is_ok(),
                "Failed to generate token pair for email: {}",
                email_str
            );

            let token_pair = result.unwrap();
            let claims = validate_token(&token_pair.access_token).await.unwrap();
            assert_eq!(claims.sub, email_str);
        }
    }

    #[tokio::test]
    async fn test_token_pair_structure() {
        let fake_email: String = SafeEmail().fake();
        let email = Email::new(fake_email.clone()).unwrap();

        let token_pair = generate_token_pair(&email).unwrap();

        let access_claims = validate_token(&token_pair.access_token).await.unwrap();
        assert_eq!(access_claims.sub, fake_email);
        assert_eq!(access_claims.token_type, "access");
        assert!(access_claims.exp > 0);

        let refresh_claims = validate_token(&token_pair.refresh_token).await.unwrap();
        assert_eq!(refresh_claims.sub, fake_email);
        assert_eq!(refresh_claims.token_type, "refresh");
        assert!(refresh_claims.exp > 0);

        assert!(refresh_claims.exp > access_claims.exp);
    }
}
