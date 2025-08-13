use crate::app_state::AppState;
use crate::domain::{AuthAPIError, Email};
use crate::utils::{generate_auth_cookie, generate_refresh_cookie};
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum_extra::extract::CookieJar;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Verify2FARequest {
    pub email: String,

    #[serde(rename = "loginAttemptId")]
    pub login_attempt_id: String,

    #[serde(rename = "2FACode")]
    pub two_fa_code: String,
}

pub async fn verify_2fa(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(request): Json<Verify2FARequest>,
) -> Result<(CookieJar, impl IntoResponse), AuthAPIError> {
    let email = validate_email(&request.email)?;
    let login_attempt_id = validate_login_attempt_id(&request.login_attempt_id)?;
    let two_fa_code = validate_two_fa_code(&request.two_fa_code)?;
    let two_fa_code_store = state.two_fa_code_store.read().await;

    let email = &Email::new(email.to_owned())?;
    let stored_data = two_fa_code_store
        .get_code(&email)
        .await
        .map_err(|_| AuthAPIError::IncorrectCredentials)?;
    let (stored_login_attempt, stored_two_fa_code) = stored_data;

    if stored_login_attempt.clone().id() != login_attempt_id {
        return Err(AuthAPIError::IncorrectCredentials);
    }

    if stored_two_fa_code.clone().code() != two_fa_code {
        return Err(AuthAPIError::IncorrectCredentials);
    }

    Ok((
        jar.add(generate_auth_cookie(&email)?)
            .add(generate_refresh_cookie(&email)?),
        StatusCode::OK.into_response(),
    ))
}

fn validate_email(email: &str) -> Result<&str, AuthAPIError> {
    if email.is_empty() || !email.contains('@') {
        return Err(AuthAPIError::EmailOrPasswordIncorrect);
    }
    Ok(email)
}

fn validate_login_attempt_id(login_attempt_id: &str) -> Result<&str, AuthAPIError> {
    if login_attempt_id.is_empty() {
        return Err(AuthAPIError::LoginAttemptIdMalformedError);
    }

    match uuid::Uuid::parse_str(login_attempt_id) {
        Ok(_) => Ok(login_attempt_id),
        Err(_) => Err(AuthAPIError::LoginAttemptIdMalformedError),
    }
}

fn validate_two_fa_code(two_fa_code: &str) -> Result<&str, AuthAPIError> {
    if two_fa_code.len() != 6 {
        return Err(AuthAPIError::TwoFAMalformedError);
    }

    if !two_fa_code.chars().all(|c| c.is_ascii_digit()) {
        return Err(AuthAPIError::TwoFAMalformedError);
    }

    Ok(two_fa_code)
}
