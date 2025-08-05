use crate::app_state::AppState;
use crate::domain::AuthAPIError;
use crate::utils::{validate_token, JWT_COOKIE_NAME};
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum_extra::extract::cookie::Cookie;
use axum_extra::extract::CookieJar;

pub async fn logout(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<(CookieJar, impl IntoResponse), AuthAPIError> {
    match jar.get(JWT_COOKIE_NAME) {
        Some(cookie) => {
            validate_token(&cookie.value().to_string())
                .await
                .map_err(|_| AuthAPIError::TokenNotValid)?;
            match state
                .banned_token_store
                .write()
                .await
                .store_token(&cookie.value().to_string())
                .await
            {
                Ok(_) => {
                    let jar = jar.remove(Cookie::build(JWT_COOKIE_NAME));
                    Ok((jar, StatusCode::OK.into_response()))
                }
                Err(_) => Err(AuthAPIError::ErrorAddingToBannedTokens),
            }
        }
        None => Err(AuthAPIError::MissingToken),
    }
}
