use crate::domain::AuthAPIError;
use axum::Json;
use axum::http::StatusCode;
use axum::response::IntoResponse;
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
    Json(request): Json<Verify2FARequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    let email = validate_email(&request.email)?;
    let login_attempt_id = validate_login_attempt_id(&request.login_attempt_id)?;
    let two_fa_code = validate_two_fa_code(&request.two_fa_code)?;

    Ok(StatusCode::OK.into_response())
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
