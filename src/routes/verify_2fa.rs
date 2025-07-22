use axum::http::StatusCode;
use axum::response::IntoResponse;

pub async fn verify_2fa() -> impl IntoResponse {
    StatusCode::OK.into_response()
}
