use crate::helpers::TestApp;
use fake::Fake;
use fake::faker::internet::en::{Password as FakePassword, SafeEmail};
use reqwest::StatusCode;

#[tokio::test]
async fn should_return_204_if_deleted_successfully() {
    let app = TestApp::new().await;
    let fake_email: String = SafeEmail().fake();
    let fake_password: String = FakePassword(8..20).fake();

    let tc = serde_json::json!({
        "email": fake_email.clone(),
        "password": fake_password,
        "requires2FA": true
    });

    let response1 = app.post_signup(&tc).await;
    assert_eq!(response1.status().as_u16(), StatusCode::CREATED);

    let rc = serde_json::json!({
        "email": fake_email,
    });

    let delete_response = app.delete_account(&rc).await;
    assert_eq!(delete_response.status().as_u16(), StatusCode::NO_CONTENT);
}
