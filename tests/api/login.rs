use crate::helpers::TestApp;

#[tokio::test]
async fn test_get_login() {
    let app = TestApp::new().await;
    let response = app.get_login().await;

    assert_eq!(response.status(), reqwest::StatusCode::OK);
}
