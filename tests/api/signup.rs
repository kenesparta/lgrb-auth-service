use crate::helpers::{get_random_email, TestApp};
use auth_service::routes::SignupResponse;
use auth_service::ErrorResponse;
use reqwest::StatusCode;

#[tokio::test]
async fn should_return_422_if_malformed_input() {
    let app = TestApp::new().await;

    let test_cases = [
        serde_json::json!({
            "email": get_random_email(),
            "password": "password123",
            "2FA": true
        }),
        serde_json::json!({
            "email": get_random_email(),
            "passwords": "123123",
            "requires2FA": false
        }),
    ];

    for test_case in test_cases.iter() {
        let response = app.post_signup(test_case).await;
        assert_eq!(
            response.status().as_u16(),
            StatusCode::UNPROCESSABLE_ENTITY,
            "Failed for input: {:?}",
            test_case
        );
    }
}

#[tokio::test]
async fn should_return_201_if_valid_input() {
    let app = TestApp::new().await;
    let response = app
        .post_signup(&serde_json::json!({
            "email": get_random_email(),
            "password": "password123",
            "requires2FA": false
        }))
        .await;

    assert_eq!(response.status().as_u16(), StatusCode::CREATED);

    let expected_response = SignupResponse {
        message: "User created successfully!".to_string(),
    };

    assert_eq!(
        response.json::<SignupResponse>().await.unwrap(),
        expected_response
    );
}

#[tokio::test]
async fn should_return_400_if_invalid_input() {
    let app = TestApp::new().await;

    let test_cases = [
        serde_json::json!({
            "email": get_random_email(),
            "password": "",
            "requires2FA": true
        }),
        serde_json::json!({
            "email": "1234example.com",
            "password": "123123",
            "requires2FA": false
        }),
    ];

    for test_case in test_cases.iter() {
        let response = app.post_signup(test_case).await;
        assert_eq!(
            response.status().as_u16(),
            StatusCode::BAD_REQUEST,
            "Failed for input: {:?}",
            test_case
        );

        assert_eq!(
            response
                .json::<ErrorResponse>()
                .await
                .expect("")
                .error_message,
            "Invalid credentials"
        );
    }
}

#[tokio::test]
async fn should_return_409_if_email_already_exists() {
    let app = TestApp::new().await;

    let tc = serde_json::json!({
        "email": get_random_email(),
        "password": "8FNowQhkKHcxzJg6",
        "requires2FA": false
    });

    let response1 = app.post_signup(&tc).await;
    assert_eq!(response1.status().as_u16(), StatusCode::CREATED);

    let response2 = app.post_signup(&tc).await;
    assert_eq!(response2.status().as_u16(), StatusCode::CONFLICT);
    assert_eq!(
        response2
            .json::<ErrorResponse>()
            .await
            .expect("")
            .error_message,
        "User already exists"
    );
}
