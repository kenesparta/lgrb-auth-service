use crate::helpers::TestApp;

#[tokio::test]
async fn test_get_logout() {
    let app = TestApp::new().await;
    let response = app.get_logout().await;

    assert_eq!(response.status(), reqwest::StatusCode::OK);
}
