use crate::domain::{AuthAPIError, Email};
use crate::utils::{
    JWT_REFRESH_COOKIE_NAME, generate_auth_cookie, generate_refresh_cookie, generate_token_pair,
    validate_token,
};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshTokenResponse {
    pub message: String,
    pub access_token: String,
    pub refresh_token: String,
}

pub async fn refresh_token(jar: CookieJar) -> Result<(CookieJar, impl IntoResponse), AuthAPIError> {
    let refresh_cookie = jar
        .get(JWT_REFRESH_COOKIE_NAME)
        .ok_or(AuthAPIError::MissingToken)?;

    let refresh_token = refresh_cookie.value();

    let claims = validate_token(refresh_token)
        .await
        .map_err(|_| AuthAPIError::TokenNotValid)?;

    if claims.token_type != "refresh" {
        return Err(AuthAPIError::TokenNotValid);
    }

    let email = &Email::new(claims.sub)?;
    let new_token_pair = generate_token_pair(&email)?;

    let response = RefreshTokenResponse {
        message: "Tokens refreshed successfully".to_string(),
        access_token: new_token_pair.access_token,
        refresh_token: new_token_pair.refresh_token,
    };

    Ok((
        jar.add(generate_auth_cookie(&email)?)
            .add(generate_refresh_cookie(&email)?),
        (StatusCode::OK, axum::Json(response)).into_response(),
    ))
}
