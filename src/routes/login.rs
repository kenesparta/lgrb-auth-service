use crate::app_state::AppState;
use crate::domain::data_stores::UserStoreError;
use crate::domain::{AuthAPIError, Email, Password};
use crate::utils::{generate_auth_cookie, generate_refresh_cookie};
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

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct LoginResponse {
    pub message: String,
}

pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(request): Json<LoginRequest>,
) -> Result<(CookieJar, impl IntoResponse), AuthAPIError> {
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

    let result = store.get_user(&email).await;
    match result {
        Ok(_) => Ok((
            jar.add(generate_auth_cookie(&email)?)
                .add(generate_refresh_cookie(&email)?),
            StatusCode::OK.into_response(),
        )),

        Err(e) => match e {
            UserStoreError::UserAlreadyExists => Err(AuthAPIError::UserAlreadyExists),
            UserStoreError::UserNotFound => Err(AuthAPIError::UnexpectedError),
            UserStoreError::IncorrectCredentials => Err(AuthAPIError::IncorrectCredentials),
            UserStoreError::UnexpectedError => Err(AuthAPIError::UnexpectedError),
        },
    }
}
