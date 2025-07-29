pub mod app_state;
mod domain;
pub mod grpc;
pub mod routes;
pub mod services;

use crate::domain::AuthAPIError;
use crate::routes::{
    delete_account, health_check, login, logout, signup, verify_2fa, verify_token,
};
use app_state::AppState;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post};
use axum::serve::Serve;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use std::error::Error;
use tower_http::services::ServeDir;

pub struct Application {
    server: Serve<Router, Router>,
    pub address: String,
}

#[derive(Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error_message: String,
}

impl IntoResponse for AuthAPIError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthAPIError::InvalidCredentials => (StatusCode::BAD_REQUEST, "Invalid credentials"),
            AuthAPIError::UnexpectedError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Unexpected error")
            }
            AuthAPIError::UserAlreadyExists => (StatusCode::CONFLICT, "User already exists"),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "Unexpected error"),
        };

        let body = Json(ErrorResponse {
            error_message: error_message.to_string(),
        });

        (status, body).into_response()
    }
}

impl Application {
    pub async fn build(app_state: AppState, address: &str) -> Result<Self, Box<dyn Error>> {
        let router = Router::new()
            .nest_service("/", ServeDir::new("assets"))
            .route("/health-check", get(health_check))
            .route("/signup", post(signup))
            .route("/delete-account", delete(delete_account))
            .route("/login", post(login))
            .route("/logout", post(logout))
            .route("/verify-2fa", post(verify_2fa))
            .route("/verify-token", post(verify_token))
            .with_state(app_state);

        let listener = tokio::net::TcpListener::bind(address).await?;
        let address = listener.local_addr()?.to_string();
        let server = axum::serve(listener, router);

        Ok(Application { server, address })
    }

    pub async fn run(self) -> Result<(), std::io::Error> {
        println!("listening on {}", &self.address);
        self.server.await
    }
}
