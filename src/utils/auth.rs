use super::constants::JWT_COOKIE_NAME;
use crate::domain::Email;
use crate::utils::{env, TOKEN_TTL_SECONDS};
use axum_extra::extract::cookie::{Cookie, SameSite};
use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::env as std_env;
use crate::utils::jwt::JWT_SECRET;

// Create a cookie with a new JWT auth token
pub fn generate_auth_cookie(email: &Email) -> Result<Cookie<'static>, GenerateTokenError> {
    let token = generate_auth_token(email)?;
    Ok(create_auth_cookie(token))
}

// Create a cookie and set the value to the passed-in token string
fn create_auth_cookie(token: String) -> Cookie<'static> {
    let subdomain =
        std_env::var(env::COOKIE_SUBDOMAIN_ENV_VAR).expect("COOKIE_SUBDOMAIN must be set.");
    let cookie = Cookie::build((JWT_COOKIE_NAME, token))
        .domain(subdomain)
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .build();

    cookie
}

#[derive(Debug, thiserror::Error)]
pub enum GenerateTokenError {
    #[error("Error creating JWT token")]
    TokenError(jsonwebtoken::errors::Error),

    #[error("Unexpected error generating JWT token")]
    UnexpectedError,
}

// Create a JWT auth token
fn generate_auth_token(email: &Email) -> Result<String, GenerateTokenError> {
    let delta = chrono::Duration::try_seconds(TOKEN_TTL_SECONDS)
        .ok_or(GenerateTokenError::UnexpectedError)?;

    // Create JWT expiration time
    let exp = Utc::now()
        .checked_add_signed(delta)
        .ok_or(GenerateTokenError::UnexpectedError)?
        .timestamp();

    // Cast exp to an usize, which is what Claims expects
    let exp: usize = exp
        .try_into()
        .map_err(|_| GenerateTokenError::UnexpectedError)?;

    let sub = email.as_ref().to_owned();

    let claims = Claims { sub, exp };

    create_token(&claims).map_err(GenerateTokenError::TokenError)
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use fake::faker::internet::en::SafeEmail;
    use fake::Fake;

    #[tokio::test]
    async fn test_generate_auth_cookie() {
        let fake_email: String = SafeEmail().fake();
        let email = Email::new(fake_email.to_owned()).unwrap();
        let cookie = generate_auth_cookie(&email).unwrap();
        assert_eq!(cookie.name(), JWT_COOKIE_NAME);
        assert_eq!(cookie.value().split('.').count(), 3);
        assert_eq!(cookie.path(), Some("/"));
        assert_eq!(cookie.http_only(), Some(true));
        assert_eq!(cookie.same_site(), Some(SameSite::Lax));
    }

    #[tokio::test]
    async fn test_create_auth_cookie() {
        let token = "test_token".to_owned();
        let cookie = create_auth_cookie(token.clone());
        assert_eq!(cookie.name(), JWT_COOKIE_NAME);
        assert_eq!(cookie.value(), token);
        assert_eq!(cookie.path(), Some("/"));
        assert_eq!(cookie.http_only(), Some(true));
        assert_eq!(cookie.same_site(), Some(SameSite::Lax));
    }

    #[tokio::test]
    async fn test_generate_auth_token() {
        let fake_email: String = SafeEmail().fake();
        let email = Email::new(fake_email.to_owned()).unwrap();
        let result = generate_auth_token(&email).unwrap();
        assert_eq!(result.split('.').count(), 3);
    }

    #[tokio::test]
    async fn test_validate_token_with_valid_token() {
        let fake_email: String = SafeEmail().fake();
        let email = Email::new(fake_email.clone().to_owned()).unwrap();
        let token = generate_auth_token(&email).unwrap();
        let result = validate_token(&token).await.unwrap();
        assert_eq!(result.sub, fake_email);

        let exp = Utc::now()
            .checked_add_signed(chrono::Duration::try_minutes(9).expect("valid duration"))
            .expect("valid timestamp")
            .timestamp();

        assert!(result.exp > exp as usize);
    }

    #[tokio::test]
    async fn test_validate_token_with_invalid_token() {
        let token = "invalid_token".to_owned();
        let result = validate_token(&token).await;
        assert!(result.is_err());
    }
}
