use crate::helpers::TestApp;
use auth_service::utils::JWT_COOKIE_NAME;
use fake::Fake;
use fake::faker::internet::en::{Password as FakePassword, SafeEmail};
use reqwest::{StatusCode, Url};

#[tokio::test]
async fn should_return_400_if_jwt_cookie_missing() {
    let mut app = TestApp::new().await;
    let response = app.post_logout().await;
    assert_eq!(response.status().as_u16(), StatusCode::BAD_REQUEST);
    app.clean_up().await;
}

#[tokio::test]
async fn should_return_401_if_invalid_token() {
    let mut app = TestApp::new().await;

    app.cookie_jar.add_cookie_str(
        &format!(
            "{}=invalid; HttpOnly; SameSite=Lax; Secure; Path=/",
            JWT_COOKIE_NAME
        ),
        &Url::parse("http://127.0.0.1").expect("Failed to parse URL"),
    );

    let response = app.post_logout().await;
    assert_eq!(response.status().as_u16(), StatusCode::UNAUTHORIZED);

    app.clean_up().await;
}

#[tokio::test]
async fn should_return_200_if_valid_jwt_cookie() {
    let mut app = TestApp::new().await;
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

    let auth_cookie = response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");

    let response = app.post_logout().await;
    assert_eq!(response.status().as_u16(), StatusCode::OK);

    {
        let banned_tokens = app.banned_tokens.read().await;
        assert!(banned_tokens.is_banned(&auth_cookie.value()).await.unwrap());
    }

    app.clean_up().await;
}

#[tokio::test]
async fn should_return_400_if_logout_called_twice_in_a_row() {
    let mut app = TestApp::new().await;
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

    app.clean_up().await;
}
