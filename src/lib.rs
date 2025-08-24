pub mod app_state;
pub mod domain;
pub mod grpc;
pub mod routes;
pub mod services;
pub mod utils;

use crate::domain::AuthAPIError;
use crate::routes::{
    delete_account, health_check, login, logout, refresh_token, signup, verify_2fa,
};
use crate::utils::{
    CORS_ALLOWED_ORIGINS, PGSQL_MAX_CONNECTIONS, make_span_with_request_id, on_request, on_response,
};
use app_state::AppState;
use axum::http::{Method, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post};
use axum::serve::Serve;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::error::Error;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

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
            AuthAPIError::IncorrectCredentials => {
                (StatusCode::UNAUTHORIZED, "Incorrect credentials")
            }
            AuthAPIError::UnexpectedError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Unexpected error")
            }
            AuthAPIError::UserAlreadyExists => (StatusCode::CONFLICT, "User already exists"),
            AuthAPIError::EmailOrPasswordIncorrect => {
                (StatusCode::BAD_REQUEST, "Email or password incorrect")
            }
            AuthAPIError::MissingToken => (StatusCode::BAD_REQUEST, "Missing JWT token"),
            AuthAPIError::TokenNotValid => (StatusCode::UNAUTHORIZED, "JWT token not valid"),
            AuthAPIError::ErrorAddingToBannedTokens => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error adding to banned tokens",
            ),
            AuthAPIError::TwoFAMalformedError => (
                StatusCode::BAD_REQUEST,
                "Error two-factor authentication malformed",
            ),
            AuthAPIError::LoginAttemptIdMalformedError => {
                (StatusCode::BAD_REQUEST, "Error login attempt id malformed")
            }
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
            .route("/refresh-token", post(refresh_token))
            .with_state(app_state)
            .layer(cors()?)
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(make_span_with_request_id)
                    .on_request(on_request)
                    .on_response(on_response),
            );

        let listener = tokio::net::TcpListener::bind(address).await?;
        let address = listener.local_addr()?.to_string();
        let server = axum::serve(listener, router);

        Ok(Application { server, address })
    }

    pub async fn run(self) -> Result<(), std::io::Error> {
        tracing::info!("listening on {}", &self.address);
        self.server.await
    }
}

fn cors() -> Result<CorsLayer, Box<dyn Error>> {
    let allowed_origins = &CORS_ALLOWED_ORIGINS;
    let origins: Result<Vec<_>, _> = allowed_origins
        .split(',')
        .map(|origin| origin.trim().parse())
        .collect();

    Ok(CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::DELETE, Method::PUT])
        .allow_credentials(true)
        .allow_origin(origins?))
}

pub async fn get_postgres_pool(url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(PGSQL_MAX_CONNECTIONS)
        .connect(url)
        .await
}

pub fn get_redis_client(redis_hostname: String) -> redis::RedisResult<redis::Client> {
    let redis_url = format!("redis://{}/", redis_hostname);
    redis::Client::open(redis_url)
}
