use crate::helpers::TestApp;
use auth_service::utils::JWT_COOKIE_NAME;
use fake::Fake;
use fake::faker::internet::en::{Password as FakePassword, SafeEmail};
use reqwest::StatusCode;
use auth_service::routes::TwoFactorAuthResponse;

#[tokio::test]
async fn should_return_422_if_malformed_credentials() {
    let app = TestApp::new().await;
    let fake_email: String = SafeEmail().fake();
    let fake_password: String = FakePassword(8..20).fake();

    let test_cases = [
        serde_json::json!({
            "emails": fake_email,
            "password": fake_password,
        }),
        serde_json::json!({
            "email": fake_email,
            "passwords": fake_password,
        }),
    ];

    for test_case in test_cases.iter() {
        let response = app.post_login(test_case).await;
        assert_eq!(
            response.status().as_u16(),
            StatusCode::UNPROCESSABLE_ENTITY,
            "Failed for input: {:?}",
            test_case
        );
    }
}

#[tokio::test]
async fn should_return_400_if_invalid_input() {
    let app = TestApp::new().await;
    let fake_password: String = FakePassword(8..20).fake();

    let test_cases = [serde_json::json!({
        "email": "1234example.com",
        "password": fake_password,
    })];

    for test_case in test_cases.iter() {
        let response = app.post_login(test_case).await;
        assert_eq!(
            response.status().as_u16(),
            StatusCode::BAD_REQUEST,
            "Failed for input: {:?}",
            test_case
        );
    }
}

#[tokio::test]
async fn should_return_401_if_incorrect_credentials() {
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

    let fake_incorrect_pass: String = FakePassword(8..20).fake();
    let case = &serde_json::json!({
        "email": fake_email,
        "password": fake_incorrect_pass,
    });

    let response_login = app.post_login(case);
    assert_eq!(
        response_login.await.status().as_u16(),
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn should_return_200_if_valid_credentials_and_2fa_disabled() {
    let app = TestApp::new().await;

    let fake_email: String = SafeEmail().fake();
    let fake_password: String = FakePassword(8..20).fake();
    let signup_body = serde_json::json!({
        "email": fake_email.clone(),
        "password": fake_password.clone(),
        "requires2FA": false
    });
    let response = app.post_signup(&signup_body).await;
    assert_eq!(response.status().as_u16(), 201);

    let login_body = serde_json::json!({
        "email": fake_email,
        "password": fake_password.clone(),
    });
    let response = app.post_login(&login_body).await;
    assert_eq!(response.status().as_u16(), 200);

    let auth_cookie = response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");
    assert!(!auth_cookie.value().is_empty());
}

#[tokio::test]
async fn should_return_206_if_valid_credentials_and_2fa_enabled() {
    let app = TestApp::new().await;

    let fake_email: String = SafeEmail().fake();
    let fake_password: String = FakePassword(8..20).fake();

    let signup_body = serde_json::json!({
        "email": fake_email.clone(),
        "password": fake_password.clone(),
        "requires2FA": true
    });
    let response = app.post_signup(&signup_body).await;
    assert_eq!(response.status().as_u16(), 201);


    let login_body = serde_json::json!({
        "email": fake_email,
        "password": fake_password.clone(),
    });
    let response = app.post_login(&login_body).await;
    assert_eq!(
        response
            .json::<TwoFactorAuthResponse>()
            .await
            .expect("Could not deserialize response body to TwoFactorAuthResponse")
            .message,
        "2FA required".to_owned()
    );
}