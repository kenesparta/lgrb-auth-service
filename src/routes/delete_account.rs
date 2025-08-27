use crate::app_state::AppState;
use crate::domain::{AuthAPIError, Email};
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use color_eyre::eyre::eyre;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct DeleteRequest {
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct DeleteResponse {
    pub message: String,
}

#[tracing::instrument(name = "DeleteAccount", skip_all)]
pub async fn delete_account(
    State(state): State<AppState>,
    Json(request): Json<DeleteRequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    let mut user_store = state.user_store.write().await;

    let email = Email::new(request.email)?;
    match user_store.delete_account(&email).await {
        Ok(_) => {}
        Err(_) => {
            return Err(AuthAPIError::UnexpectedError(eyre!(
                "Unexpected error deleting an account"
            )));
        }
    }

    let response = Json(DeleteResponse {
        message: "Account deleted".to_string(),
    });

    Ok((StatusCode::NO_CONTENT, response))
}
