use crate::app_state::AppState;
use crate::domain::data_stores::UserStoreError;
use crate::domain::{AuthAPIError, Email, Password};
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
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
    Json(request): Json<LoginRequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
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
        Ok(_) => Ok((StatusCode::OK, ())),

        Err(e) => match e {
            UserStoreError::UserAlreadyExists => Err(AuthAPIError::UserAlreadyExists),
            UserStoreError::UserNotFound => Err(AuthAPIError::UnexpectedError),
            UserStoreError::IncorrectCredentials => Err(AuthAPIError::IncorrectCredentials),
            UserStoreError::UnexpectedError => Err(AuthAPIError::UnexpectedError),
        },
    }
}
