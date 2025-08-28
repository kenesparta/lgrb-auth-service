use crate::helpers::TestApp;
use auth_service::domain::Email;
use auth_service::utils::JWT_COOKIE_NAME;
use fake::Fake;
use fake::faker::internet::en::{Password as FakePassword, SafeEmail};
use fake::faker::number::en::NumberWithFormat;
use reqwest::StatusCode;
use secrecy::{ExposeSecret, SecretBox};
use uuid::Uuid;

#[tokio::test]
async fn should_return_422_if_malformed_input() {
    let mut app = TestApp::new().await;
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

    app.clean_up().await;
}

#[tokio::test]
async fn should_return_400_if_invalid_input() {
    let mut app = TestApp::new().await;
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

    app.clean_up().await;
}

#[tokio::test]
async fn should_return_401_if_incorrect_credentials() {
    let mut app = TestApp::new().await;
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

    let case = &serde_json::json!({
        "email": fake_email,
        "password": fake_password,
    });

    let response_login = app.post_login(case);
    assert_eq!(response_login.await.status().as_u16(), StatusCode::PARTIAL_CONTENT);

    let new_fake_email: String = SafeEmail().fake();
    let case = &serde_json::json!({
        "email": new_fake_email,
        "loginAttemptId": Uuid::new_v4().to_string(),
        "2FACode": "123456",
    });

    let response_2fa = app.post_verify_2fa(case);
    assert_eq!(response_2fa.await.status().as_u16(), StatusCode::UNAUTHORIZED);

    app.clean_up().await;
}

#[tokio::test]
async fn should_return_401_if_old_code() {
    let mut app = TestApp::new().await;
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

    let first_response = app
        .post_login(&serde_json::json!({
            "email": fake_email,
            "password": fake_password,
        }))
        .await;
    assert_eq!(first_response.status().as_u16(), StatusCode::PARTIAL_CONTENT);

    let email = &Email::new(SecretBox::new(Box::from(fake_email.clone()))).unwrap();
    let two_fa_store = {
        let guard = app.two_fa_code.read().await;
        guard.get_code(&email).await.unwrap().clone()
    };

    let second_response = app
        .post_login(&serde_json::json!({
            "email": fake_email,
            "password": fake_password,
        }))
        .await;
    assert_eq!(second_response.status().as_u16(), StatusCode::PARTIAL_CONTENT);

    let case = &serde_json::json!({
        "email": fake_email,
        "loginAttemptId": two_fa_store.clone().0.id().expose_secret(),
        "2FACode": two_fa_store.clone().1.code(),
    });
    let response_2fa = app.post_verify_2fa(case);
    assert_eq!(response_2fa.await.status().as_u16(), StatusCode::UNAUTHORIZED);

    app.clean_up().await;
}

#[tokio::test]
async fn should_return_200_if_correct_code() {
    let mut app = TestApp::new().await;
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

    let first_response = app
        .post_login(&serde_json::json!({
            "email": fake_email,
            "password": fake_password,
        }))
        .await;
    assert_eq!(first_response.status().as_u16(), StatusCode::PARTIAL_CONTENT);

    let email = &Email::new(SecretBox::new(Box::from(fake_email.clone()))).unwrap();
    let two_fa_store = {
        let guard = app.two_fa_code.read().await;
        guard.get_code(&email).await.unwrap().clone()
    };

    let case = &serde_json::json!({
        "email": fake_email,
        "loginAttemptId": two_fa_store.clone().0.id().expose_secret(),
        "2FACode": two_fa_store.clone().1.code(),
    });
    let response_2fa = app.post_verify_2fa(case).await;
    assert_eq!(response_2fa.status().as_u16(), StatusCode::OK);

    let auth_cookie = response_2fa
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");
    assert!(!auth_cookie.value().is_empty());

    app.clean_up().await;
}

#[tokio::test]
async fn should_return_401_if_same_code_twice() {
    let mut app = TestApp::new().await;
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

    let first_response = app
        .post_login(&serde_json::json!({
            "email": fake_email,
            "password": fake_password,
        }))
        .await;
    assert_eq!(first_response.status().as_u16(), StatusCode::PARTIAL_CONTENT);

    let email = &Email::new(SecretBox::new(Box::from(fake_email.clone()))).unwrap();
    let two_fa_store = {
        let guard = app.two_fa_code.read().await;
        guard.get_code(&email).await.unwrap().clone()
    };

    let case = &serde_json::json!({
        "email": fake_email,
        "loginAttemptId": two_fa_store.clone().0.id().expose_secret(),
        "2FACode": two_fa_store.clone().1.code(),
    });
    let response_2fa = app.post_verify_2fa(case).await;
    assert_eq!(response_2fa.status().as_u16(), StatusCode::OK);

    let case = &serde_json::json!({
        "email": fake_email,
        "loginAttemptId": two_fa_store.clone().0.id().expose_secret(),
        "2FACode": two_fa_store.clone().1.code(),
    });
    let response_2fa = app.post_verify_2fa(case).await;
    assert_eq!(response_2fa.status().as_u16(), StatusCode::UNAUTHORIZED);

    app.clean_up().await;
}
