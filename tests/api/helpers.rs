use auth_service::app_state::{AppState, BannedTokenStoreType, TwoFACodeStoreType};
use auth_service::services::MockEmailClient;
use auth_service::services::data_stores::{PostgresUserStore, RedisBannedTokenStore, RedisTwoFACodeStore};
use auth_service::utils::{DATABASE_URL, REDIS_HOST_NAME, test};
use auth_service::{Application, get_postgres_pool, get_redis_client};
use reqwest::cookie::Jar;
use secrecy::{ExposeSecret, SecretBox};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

pub struct TestApp {
    pub address: String,
    pub http_client: reqwest::Client,
    pub cookie_jar: Arc<Jar>,
    pub banned_tokens: BannedTokenStoreType,
    pub two_fa_code: TwoFACodeStoreType,
    pub clean_up_called: bool,
    pub db_name: String,
}

impl TestApp {
    pub async fn new() -> Self {
        let db_name = Uuid::new_v4().to_string();
        let pg_pool = configure_postgresql(db_name.as_str()).await;
        let redis_conn = configure_redis().await;

        let user_store = Arc::new(RwLock::new(PostgresUserStore::new(pg_pool)));
        let banned_tokens: BannedTokenStoreType = Arc::new(RwLock::new(RedisBannedTokenStore::new(redis_conn.clone())));
        let two_fa_code = Arc::new(RwLock::new(RedisTwoFACodeStore::new(redis_conn)));
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
        let clean_up_called = false;

        TestApp {
            address,
            http_client,
            cookie_jar,
            banned_tokens,
            two_fa_code,
            clean_up_called,
            db_name,
        }
    }

    pub async fn post_signup<Body>(
        &self,
        body: &Body,
    ) -> reqwest::Response
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

    pub async fn post_login<Body>(
        &self,
        body: &Body,
    ) -> reqwest::Response
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

    pub async fn post_verify_2fa<Body>(
        &self,
        body: &Body,
    ) -> reqwest::Response
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

    pub async fn delete_account<Body>(
        &self,
        body: &Body,
    ) -> reqwest::Response
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

    pub async fn clean_up(&mut self) {
        delete_database(&self.db_name).await;
        self.clean_up_called = true;
    }
}

impl Drop for TestApp {
    fn drop(&mut self) {
        if self.clean_up_called {
            return;
        }

        let db_name = self.db_name.clone();
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            handle.spawn(async move {
                delete_database(&db_name).await;
            });
        } else {
            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime for cleanup");
                rt.block_on(async move {
                    delete_database(&db_name).await;
                });
            });
        }
        self.clean_up_called = true;
    }
}

async fn configure_postgresql(db_name: &str) -> PgPool {
    configure_database(&DATABASE_URL, &db_name).await;

    let postgresql_conn_url_with_db = format!("{}/{}", &DATABASE_URL.expose_secret().as_str(), db_name);
    get_postgres_pool(&SecretBox::new(Box::from(postgresql_conn_url_with_db)))
        .await
        .expect("Failed to create a Postgres connection pool!")
}

async fn configure_database(
    db_conn_string: &SecretBox<String>,
    db_name: &str,
) {
    let connection = PgPoolOptions::new()
        .connect(db_conn_string.expose_secret())
        .await
        .expect("Failed to create a Postgres connection pool.");

    // Create a new database
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, db_name).as_str())
        .await
        .expect("Failed to create a database.");

    // Connect to a new database
    let db_conn_string = format!("{}/{}", db_conn_string.expose_secret(), db_name);

    let connection = PgPoolOptions::new()
        .connect(&db_conn_string)
        .await
        .expect("Failed to create a Postgres connection pool.");

    sqlx::migrate!()
        .run(&connection)
        .await
        .expect("Failed to migrate the database");
}

async fn delete_database(db_name: &str) {
    let connection_options = PgConnectOptions::from_str(&DATABASE_URL.expose_secret())
        .expect("Failed to parse PostgreSQL connection string");

    let mut connection = PgConnection::connect_with(&connection_options)
        .await
        .expect("Failed to connect to Postgres");

    // Kill any active connections to the database
    connection
        .execute(
            format!(
                r#"
                SELECT pg_terminate_backend(pg_stat_activity.pid)
                FROM pg_stat_activity
                WHERE pg_stat_activity.datname = '{}'
                  AND pid <> pg_backend_pid();
        "#,
                db_name
            )
            .as_str(),
        )
        .await
        .expect("Failed to drop the database.");

    // Drop the database
    connection
        .execute(format!(r#"DROP DATABASE "{}";"#, db_name).as_str())
        .await
        .expect("Failed to drop the database.");
}

async fn configure_redis() -> redis::aio::MultiplexedConnection {
    let client = get_redis_client(REDIS_HOST_NAME.to_owned()).expect("Failed to get a Redis client");

    client
        .get_multiplexed_async_connection()
        .await
        .expect("Failed to create Redis connection manager")
}
