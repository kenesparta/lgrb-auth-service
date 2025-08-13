use crate::helpers::TestApp;
use fake::Fake;
use fake::faker::internet::en::{Password as FakePassword, SafeEmail};
use fake::faker::number::en::NumberWithFormat;
use reqwest::StatusCode;
use uuid::Uuid;

#[tokio::test]
async fn should_return_422_if_malformed_input() {
    let app = TestApp::new().await;
    let fake_email: String = SafeEmail().fake();
    let uuid_random: String = Uuid::new_v4().to_string();
    let fake_2fa_code: String = NumberWithFormat("######").fake();

    let test_cases = [serde_json::json!({
        "emails": fake_email,
        "login_attempt_id": uuid_random,
        "two_fa_codes": fake_2fa_code,
    })];

    for test_case in test_cases.iter() {
        let response = app.post_verify_2fa(test_case).await;
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
    let fake_email: String = SafeEmail().fake();
    let uuid_random: String = Uuid::new_v4().to_string();
    let fake_2fa_code: String = NumberWithFormat("######").fake();

    let test_cases = [
        serde_json::json!({
            "email": "1234567890",
            "loginAttemptId": uuid_random,
            "2FACode": fake_2fa_code,
        }),
        serde_json::json!({
            "email": fake_email,
            "loginAttemptId": uuid_random,
            "2FACode": "12a",
        }),
        serde_json::json!({
            "email": fake_email,
            "loginAttemptId": "abc-123",
            "2FACode": fake_2fa_code,
        }),
    ];

    for test_case in test_cases.iter() {
        let response = app.post_verify_2fa(test_case).await;
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
async fn should_return_401_if_old_code() {
    let app = TestApp::new().await;
    let fake_email: String = SafeEmail().fake();
    let fake_password: String = FakePassword(8..20).fake();

    let response = app
        .post_signup(&serde_json::json!({
            "email": fake_email,
            "password": fake_password,
            "requires2FA": true
        }))
        .await;
    assert_eq!(response.status().as_u16(), StatusCode::CREATED);

    let response = app
        .post_login(&serde_json::json!({
            "email": fake_email,
            "password": fake_password,
        }))
        .await;
    assert_eq!(response.status().as_u16(), StatusCode::PARTIAL_CONTENT);

    // let response = app.post_logout().await;
    // assert_eq!(response.status().as_u16(), StatusCode::OK);
    //
    // let response = app.post_logout().await;
    // assert_eq!(response.status().as_u16(), StatusCode::BAD_REQUEST);
}
