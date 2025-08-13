use crate::app_state::AppState;
use crate::domain::data_stores::UserStoreError;
use crate::domain::{AuthAPIError, Email, LoginAttemptId, Password, TwoFACode};
use crate::utils::{email, generate_auth_cookie, generate_refresh_cookie};
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

// The login route can return 2 possible success responses.
// This enum models each response!
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum LoginResponse {
    RegularAuth,
    TwoFactorAuth(TwoFactorAuthResponse),
}

// If a user requires 2FA, this JSON body should be returned!
#[derive(Debug, Serialize, Deserialize)]
pub struct TwoFactorAuthResponse {
    pub message: String,
    #[serde(rename = "loginAttemptId")]
    pub login_attempt_id: String,
}

pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(request): Json<LoginRequest>,
) -> Result<(StatusCode, CookieJar, impl IntoResponse), AuthAPIError> {
    // TODO: move this validation other part of the code
    if request.email.is_empty() || request.password.len() < 8 || !request.email.contains('@') {
        return Err(AuthAPIError::EmailOrPasswordIncorrect);
    }

    let store = &state.user_store.read().await;
    let email = &Email::new(request.email)?;
    let password = &Password::new(request.password)?;
    match store.validate_user(email, password).await {
        Ok(_) => (),
        Err(_) => return Err(AuthAPIError::IncorrectCredentials),
    }

    let user = match store.get_user(email).await {
        Ok(user) => user,
        Err(e) => {
            return match e {
                UserStoreError::UserAlreadyExists => Err(AuthAPIError::UserAlreadyExists),
                UserStoreError::UserNotFound => Err(AuthAPIError::UnexpectedError),
                UserStoreError::IncorrectCredentials => Err(AuthAPIError::IncorrectCredentials),
                UserStoreError::UnexpectedError => Err(AuthAPIError::UnexpectedError),
            };
        }
    };

    match user.requires_2fa() {
        true => handle_2fa(&user.email(), &state, jar).await,
        false => handle_no_2fa(&user.email(), jar).await,
    }
}

async fn handle_2fa(
    email: &Email,
    state: &AppState,
    jar: CookieJar,
) -> Result<(StatusCode, CookieJar, Json<LoginResponse>), AuthAPIError> {
    let login_attempt_id = LoginAttemptId::default();
    let two_fa_code = TwoFACode::default();

    match state
        .email_client
        .read()
        .await
        .send_email(
            email,
            email::SUBJECT,
            format!("your code is here: {}", two_fa_code.clone()).as_str(),
        )
        .await
    {
        Ok(_) => (),
        Err(_) => return Err(AuthAPIError::UnexpectedError),
    }

    match state
        .two_fa_code_store
        .write()
        .await
        .add_code(email, login_attempt_id.clone(), two_fa_code)
        .await
    {
        Ok(_) => Ok((
            StatusCode::PARTIAL_CONTENT,
            jar,
            Json(LoginResponse::TwoFactorAuth(TwoFactorAuthResponse {
                message: "2FA required".to_string(),
                login_attempt_id: login_attempt_id.id(),
            })),
        )),
        Err(_) => Err(AuthAPIError::UnexpectedError),
    }
}

async fn handle_no_2fa(
    email: &Email,
    jar: CookieJar,
) -> Result<(StatusCode, CookieJar, Json<LoginResponse>), AuthAPIError> {
    Ok((
        StatusCode::OK,
        jar.add(generate_auth_cookie(&email)?)
            .add(generate_refresh_cookie(&email)?),
        Json(LoginResponse::RegularAuth),
    ))
}
