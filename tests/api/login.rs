use crate::helpers::TestApp;
use fake::faker::internet::en::{Password as FakePassword, SafeEmail};
use fake::Fake;
use reqwest::StatusCode;

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
