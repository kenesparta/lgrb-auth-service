use crate::helpers::{get_random_email, TestApp};
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
