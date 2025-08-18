use auth_service::app_state::{AppState, BannedTokenStoreType, TwoFACodeStoreType};
use auth_service::services::MockEmailClient;
use auth_service::services::data_stores::{
    HashmapTwoFACodeStore, HashsetBannedTokenStore, PostgresUserStore,
};
use auth_service::utils::env::DATABASE_URL_ENV_VAR;
use auth_service::utils::test;
use auth_service::{Application, get_postgres_pool};
use reqwest::cookie::Jar;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Executor, PgPool};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

pub struct TestApp {
    pub address: String,
    pub http_client: reqwest::Client,
    pub cookie_jar: Arc<Jar>,
    pub banned_tokens: BannedTokenStoreType,
    pub two_fa_code: TwoFACodeStoreType,
}

impl TestApp {
    pub async fn new() -> Self {
        let pg_pool = configure_postgresql().await;

        let user_store = Arc::new(RwLock::new(PostgresUserStore::new(pg_pool)));
        let banned_tokens: BannedTokenStoreType =
            Arc::new(RwLock::new(HashsetBannedTokenStore::default()));
        let two_fa_code = Arc::new(RwLock::new(HashmapTwoFACodeStore::default()));
        let email_service = Arc::new(RwLock::new(MockEmailClient::new()));
        let app_state = AppState::new(
            user_store,
            banned_tokens.clone(),
            two_fa_code.clone(),
            email_service.clone(),
        );
        let app = Application::build(app_state, test::APP_ADDRESS)
            .await
            .expect("Failed to build app");

        let address = format!("http://{}", app.address.clone());
        tokio::spawn(app.run());

        let cookie_jar = Arc::new(Jar::default());
        let http_client = reqwest::Client::builder()
            .cookie_provider(cookie_jar.clone())
            .build()
            .expect("Failed to create an HTTP client");

        TestApp {
            address,
            http_client,
            cookie_jar,
            banned_tokens,
            two_fa_code,
        }
    }

    pub async fn post_signup<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(&format!("{}/signup", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute the request.")
    }

    pub async fn get_root(&self) -> reqwest::Response {
        self.http_client
            .get(&format!("{}/", &self.address))
            .send()
            .await
            .expect("Failed to execute the request.")
    }

    pub async fn post_login<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(&format!("{}/login", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute the request.")
    }

    pub async fn post_logout(&self) -> reqwest::Response {
        self.http_client
            .post(&format!("{}/logout", &self.address))
            .send()
            .await
            .expect("Failed to execute the request.")
    }

    pub async fn post_verify_2fa<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(format!("{}/verify-2fa", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute the request.")
    }

    pub async fn delete_account<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .delete(&format!("{}/delete-account", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute the request.")
    }
}

async fn configure_postgresql() -> PgPool {
    let postgresql_conn_url = std::env::var(DATABASE_URL_ENV_VAR)
        .expect("DATABASE_URL must be set in environment variables");

    // We are creating a new database for each test case, and we need to ensure each database has a unique name!
    let db_name = Uuid::new_v4().to_string();

    configure_database(&postgresql_conn_url, &db_name).await;

    let postgresql_conn_url_with_db = format!("{}/{}", postgresql_conn_url, db_name);

    // Create a new connection pool and return it
    get_postgres_pool(&postgresql_conn_url_with_db)
        .await
        .expect("Failed to create Postgres connection pool!")
}

async fn configure_database(db_conn_string: &str, db_name: &str) {
    // Create database connection
    let connection = PgPoolOptions::new()
        .connect(db_conn_string)
        .await
        .expect("Failed to create Postgres connection pool.");

    // Create a new database
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, db_name).as_str())
        .await
        .expect("Failed to create database.");

    // Connect to new database
    let db_conn_string = format!("{}/{}", db_conn_string, db_name);

    let connection = PgPoolOptions::new()
        .connect(&db_conn_string)
        .await
        .expect("Failed to create Postgres connection pool.");

    // Run migrations against new database
    sqlx::migrate!()
        .run(&connection)
        .await
        .expect("Failed to migrate the database");
}
