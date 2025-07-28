use crate::app_state::AppState;
use crate::domain::{AuthAPIError, Email, User};
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct SignupRequest {
    pub email: String,
    pub password: String,

    #[serde(rename = "requires2FA")]
    pub requires_2fa: bool,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct SignupResponse {
    pub message: String,
}

pub async fn signup(
    State(state): State<AppState>,
    Json(request): Json<SignupRequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    if request.email.is_empty() || request.password.len() < 8 || !request.email.contains('@') {
        return Err(AuthAPIError::InvalidCredentials);
    }

    let mut user_store = state.user_store.write().await;
    match user_store
        .get_user(&Email::new(request.email.clone()).unwrap())
        .await
    {
        Ok(_) => return Err(AuthAPIError::UserAlreadyExists),
        Err(_) => {}
    }

    let user = User::new(request.email, request.password, request.requires_2fa)?;

    match user_store.add_user(user).await {
        Ok(_) => {}
        Err(_) => return Err(AuthAPIError::UnexpectedError),
    }

    let response = Json(SignupResponse {
        message: "User created successfully!".to_string(),
    });

    Ok((StatusCode::CREATED, response))
}
