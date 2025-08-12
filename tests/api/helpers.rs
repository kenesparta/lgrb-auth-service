use auth_service::Application;
use auth_service::app_state::{AppState, BannedTokenStoreType, TwoFACodeStoreType, UserStoreType};
use auth_service::services::{HashmapTwoFACodeStore, HashmapUserStore, HashsetBannedTokenStore};
use auth_service::utils::test;
use reqwest::cookie::Jar;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct TestApp {
    pub address: String,
    pub http_client: reqwest::Client,
    pub cookie_jar: Arc<Jar>,
    pub banned_tokens: BannedTokenStoreType,
    pub two_fa_code: TwoFACodeStoreType,
}

impl TestApp {
    pub async fn new() -> Self {
        let user_store: UserStoreType = Arc::new(RwLock::new(HashmapUserStore::default()));
        let banned_tokens: BannedTokenStoreType =
            Arc::new(RwLock::new(HashsetBannedTokenStore::default()));
        let two_fa_code = Arc::new(RwLock::new(HashmapTwoFACodeStore::default()));
        let app_state = AppState::new(user_store, banned_tokens.clone(), two_fa_code.clone());
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

    pub async fn get_verify_2fa(&self) -> reqwest::Response {
        self.http_client
            .post(&format!("{}/verify-2fa", &self.address))
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
