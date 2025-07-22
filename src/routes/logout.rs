use axum::http::StatusCode;
use axum::response::IntoResponse;

pub async fn logout() -> impl IntoResponse {
    StatusCode::OK.into_response()
}
