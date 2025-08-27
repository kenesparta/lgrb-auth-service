use axum::http::StatusCode;
use axum::response::IntoResponse;

#[tracing::instrument(name = "HealthCheck", skip_all)]
pub async fn health_check() -> impl IntoResponse {
    StatusCode::OK.into_response()
}
