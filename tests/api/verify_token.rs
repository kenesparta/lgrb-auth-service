use auth_service::domain::Email;
use auth_service::grpc::auth_service::{
    auth_service::{VerifyTokenRequest, auth_service_client::AuthServiceClient},
    create_grpc_service,
};
use auth_service::utils::generate_auth_cookie;
use fake::Fake;
use fake::faker::internet::en::SafeEmail;
use secrecy::SecretBox;
use tonic::Request;
use tonic::transport::Server;

#[tokio::test]
async fn test_verify_token_valid() {
    let grpc_service = create_grpc_service();
    let addr = "127.0.0.1:50052".parse().unwrap();

    let server_handle = tokio::spawn(async move {
        Server::builder()
            .add_service(grpc_service)
            .serve(addr)
            .await
            .expect("Failed to start the gRPC server")
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let mut client = AuthServiceClient::connect("http://127.0.0.1:50052")
        .await
        .expect("Failed to connect to the gRPC server");

    let fake_email: String = SafeEmail().fake();
    let email = &Email::new(SecretBox::new(Box::from(fake_email))).expect("Failed to create email");
    let valid_token = generate_auth_cookie(email).expect("Failed to generate a token");

    let request = Request::new(VerifyTokenRequest {
        token: valid_token.value().to_string(),
    });

    let response = client.verify_token(request).await.expect("Request failed");
    let response = response.into_inner();

    assert!(response.valid);
    assert_eq!(response.message, "Token is valid");

    server_handle.abort();
}
