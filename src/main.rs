use auth_service::app_state::AppState;
use auth_service::grpc::auth_service::create_grpc_service;
use auth_service::services::MockEmailClient;
use auth_service::services::data_stores::{
    PostgresUserStore, RedisBannedTokenStore, RedisTwoFACodeStore,
};
use auth_service::utils::{DATABASE_URL, REDIS_HOST_NAME, prod};
use auth_service::{Application, get_postgres_pool, get_redis_client};
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::RwLock;
use tonic::transport::Server;
use tonic_reflection::server::Builder as ReflectionBuilder;

#[tokio::main]
async fn main() {
    let redis_client = configure_redis().await;
    let app_state = AppState::new(
        Arc::new(RwLock::new(PostgresUserStore::new(
            configure_postgresql().await,
        ))),
        Arc::new(RwLock::new(RedisBannedTokenStore::new(
            redis_client.clone(),
        ))),
        Arc::new(RwLock::new(RedisTwoFACodeStore::new(
            redis_client,
        ))),
        Arc::new(RwLock::new(MockEmailClient::new())),
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

async fn configure_postgresql() -> PgPool {
    let pg_pool = get_postgres_pool(&DATABASE_URL)
        .await
        .expect("Failed to create a Postgres connection pool!");

    sqlx::migrate!()
        .run(&pg_pool)
        .await
        .expect("Failed to run migrations");

    pg_pool
}

async fn configure_redis() -> redis::aio::MultiplexedConnection {
    let client =
        get_redis_client(REDIS_HOST_NAME.to_owned()).expect("Failed to get a Redis client");

    client
        .get_multiplexed_async_connection()
        .await
        .expect("Failed to create Redis connection manager")
}
