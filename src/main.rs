use auth_service::Application;
use auth_service::app_state::AppState;
use auth_service::grpc::auth_service::create_grpc_service;
use auth_service::services::{HashmapTwoFACodeStore, HashmapUserStore, HashsetBannedTokenStore};
use auth_service::utils::prod;
use std::sync::Arc;
use tokio::sync::RwLock;
use tonic::transport::Server;
use tonic_reflection::server::Builder as ReflectionBuilder;

#[tokio::main]
async fn main() {
    let app_state = AppState::new(
        Arc::new(RwLock::new(HashmapUserStore::default())),
        Arc::new(RwLock::new(HashsetBannedTokenStore::default())),
        Arc::new(RwLock::new(HashmapTwoFACodeStore::default())),
    );

    let http_app = Application::build(app_state, prod::APP_ADDRESS)
        .await
        .expect("Failed to build app");

    let grpc_service = create_grpc_service();
    let grpc_addr = "0.0.0.0:50051".parse().unwrap(); // TODO: add error handling

    let reflection = ReflectionBuilder::configure()
        .register_encoded_file_descriptor_set(include_bytes!("../proto/proto_descriptor.bin"))
        .build_v1()
        .expect("Failed to build a reflection service");

    let http_server = tokio::spawn(async move {
        http_app.run().await.expect("Failed to run HTTP app");
    });

    let grpc_server = tokio::spawn(async move {
        Server::builder()
            .add_service(grpc_service)
            .add_service(reflection)
            .serve(grpc_addr)
            .await
            .expect("Failed to run gRPC server");
    });

    tokio::select! {
        _ = http_server => println!("HTTP server finished"),
        _ = grpc_server => println!("gRPC server finished"),
    }
}
