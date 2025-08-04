use crate::helpers::TestApp;
use auth_service::utils::JWT_COOKIE_NAME;
use fake::faker::internet::en::{Password as FakePassword, SafeEmail};
use fake::Fake;
use reqwest::{StatusCode, Url};

#[tokio::test]
async fn should_return_400_if_jwt_cookie_missing() {
    let app = TestApp::new().await;
    let response = app.post_logout().await;
    assert_eq!(response.status().as_u16(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn should_return_401_if_invalid_token() {
    let app = TestApp::new().await;

    app.cookie_jar.add_cookie_str(
        &format!(
            "{}=invalid; HttpOnly; SameSite=Lax; Secure; Path=/",
            JWT_COOKIE_NAME
        ),
        &Url::parse("http://127.0.0.1").expect("Failed to parse URL"),
    );

    let response = app.post_logout().await;
    assert_eq!(response.status().as_u16(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn should_return_200_if_valid_jwt_cookie() {
    let app = TestApp::new().await;
    let fake_email: String = SafeEmail().fake();
    let fake_password: String = FakePassword(8..20).fake();

    let response = app
        .post_signup(&serde_json::json!({
            "email": fake_email.clone(),
            "password": fake_password,
            "requires2FA": false
        }))
        .await;
    assert_eq!(response.status().as_u16(), StatusCode::CREATED);

    let response = app
        .post_login(&serde_json::json!({
            "email": fake_email,
            "password": fake_password,
        }))
        .await;
    assert_eq!(response.status().as_u16(), StatusCode::OK);

    let response = app.post_logout().await;
    assert_eq!(response.status().as_u16(), StatusCode::OK);
}

#[tokio::test]
async fn should_return_400_if_logout_called_twice_in_a_row() {
    let app = TestApp::new().await;
    let fake_email: String = SafeEmail().fake();
    let fake_password: String = FakePassword(8..20).fake();

    let response = app
        .post_signup(&serde_json::json!({
            "email": fake_email.clone(),
            "password": fake_password,
            "requires2FA": false
        }))
        .await;
    assert_eq!(response.status().as_u16(), StatusCode::CREATED);

    let response = app
        .post_login(&serde_json::json!({
            "email": fake_email,
            "password": fake_password,
        }))
        .await;
    assert_eq!(response.status().as_u16(), StatusCode::OK);

    let response = app.post_logout().await;
    assert_eq!(response.status().as_u16(), StatusCode::OK);

    let response = app.post_logout().await;
    assert_eq!(response.status().as_u16(), StatusCode::BAD_REQUEST);
}
