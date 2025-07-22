use crate::helpers::TestApp;

#[tokio::test]
async fn test_get_verify_2fa() {
    let app = TestApp::new().await;
    let response = app.get_verify_2fa().await;

    assert_eq!(response.status(), reqwest::StatusCode::OK);
}
