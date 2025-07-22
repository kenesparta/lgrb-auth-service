use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct SignupRequest {
    pub email: String,
    pub password: String,

    #[serde(rename = "requires2FA")]
    pub requires_2fa: bool,
}

pub async fn signup(Json(request): Json<SignupRequest>) -> impl IntoResponse {
    StatusCode::UNAUTHORIZED.into_response()
}
