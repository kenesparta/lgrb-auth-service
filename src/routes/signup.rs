use crate::app_state::AppState;
use crate::domain::data_stores::UserStoreError;
use crate::domain::{AuthAPIError, User};
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

    let user = User::new(request.email, request.password, request.requires_2fa)?;
    let result = state.user_store.write().await.add_user(user).await;
    match result {
        Ok(()) => {
            let response = Json(SignupResponse {
                message: "User created successfully!".to_string(),
            });
            Ok((StatusCode::CREATED, response))
        }
        Err(e) => match e {
            UserStoreError::UserAlreadyExists => Err(AuthAPIError::UserAlreadyExists),
            UserStoreError::UserNotFound => Err(AuthAPIError::UnexpectedError),
            UserStoreError::InvalidCredentials => Err(AuthAPIError::InvalidCredentials),
            UserStoreError::UnexpectedError => Err(AuthAPIError::UnexpectedError),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_state::AppState;
    use crate::domain::data_stores::{MockUserStore, UserStoreError};
    use crate::domain::AuthAPIError;
    use axum::extract::State;
    use axum::Json;
    use fake::faker::internet::en::{Password as FakePassword, SafeEmail};
    use fake::Fake;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    fn create_app_state_with_mock(mock_store: MockUserStore) -> AppState {
        AppState {
            user_store: Arc::new(RwLock::new(mock_store)),
        }
    }

    #[tokio::test]
    async fn signup_unexpected_error_on_get_user() {
        let mut mock_store = MockUserStore::new();
        mock_store
            .expect_add_user()
            .times(1)
            .returning(|_| Err(UserStoreError::UnexpectedError));

        let state = create_app_state_with_mock(mock_store);
        let request = SignupRequest {
            email: SafeEmail().fake(),
            password: FakePassword(8..20).fake(),
            requires_2fa: false,
        };

        let result = signup(State(state), Json(request)).await;
        assert!(matches!(result, Err(AuthAPIError::UnexpectedError)));
    }

    #[tokio::test]
    async fn signup_fails_with_invalid_credentials_empty_email() {
        let mock_store = MockUserStore::new();
        let state = create_app_state_with_mock(mock_store);
        let request = SignupRequest {
            email: "".to_string(),
            password: FakePassword(8..20).fake(),
            requires_2fa: false,
        };
        let result = signup(State(state), Json(request)).await;
        assert!(matches!(result, Err(AuthAPIError::InvalidCredentials)));
    }

    #[tokio::test]
    async fn signup_fails_with_invalid_credentials_short_password() {
        let mock_store = MockUserStore::new();
        let state = create_app_state_with_mock(mock_store);
        let request = SignupRequest {
            email: SafeEmail().fake(),
            password: FakePassword(0..7).fake(),
            requires_2fa: false,
        };

        let result = signup(State(state), Json(request)).await;
        assert!(matches!(result, Err(AuthAPIError::InvalidCredentials)));
    }

    #[tokio::test]
    async fn signup_fails_with_invalid_credentials_no_at_symbol() {
        let mock_store = MockUserStore::new();
        let state = create_app_state_with_mock(mock_store);
        let request = SignupRequest {
            email: "testexample.com".to_string(),
            password: FakePassword(8..20).fake(),
            requires_2fa: false,
        };

        let result = signup(State(state), Json(request)).await;
        assert!(matches!(result, Err(AuthAPIError::InvalidCredentials)));
    }

    #[tokio::test]
    async fn signup_fails_with_invalid_email_format_during_user_creation() {
        let mock_store = MockUserStore::new();
        let state = create_app_state_with_mock(mock_store);
        let request = SignupRequest {
            email: "invalid@".to_string(),
            password: FakePassword(8..20).fake(),
            requires_2fa: false,
        };

        let result = signup(State(state), Json(request)).await;
        assert!(result.is_err());
    }
}
