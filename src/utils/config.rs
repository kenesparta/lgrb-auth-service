use config::{Config, ConfigError, Environment, File};
use lazy_static::lazy_static;
use secrecy::SecretBox;
use serde::{Deserialize, Serialize};
use std::sync::{LazyLock, OnceLock};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub jwt_secret: String,
    pub cookie_domain: String,
    pub database_url: String,
    pub redis_host_name: String,
    pub cors_allowed_origins: String,
    pub captcha_site_key: String,
    pub captcha_secret_key: String,
    pub postgres_password: String,
    pub token_ttl_seconds: i64,
    pub refresh_token_ttl_seconds: i64,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            jwt_secret: String::new(),
            cookie_domain: String::new(),
            database_url: String::new(),
            redis_host_name: "127.0.0.1".to_string(),
            cors_allowed_origins: "http://127.0.0.1,http://localhost".to_string(),
            captcha_site_key: String::new(),
            captcha_secret_key: String::new(),
            postgres_password: String::new(),
            token_ttl_seconds: 600,
            refresh_token_ttl_seconds: 3600,
        }
    }
}

impl AppConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let config = Config::builder()
            .add_source(Config::try_from(&AppConfig::default())?)
            .add_source(File::with_name("config.yaml").required(false))
            .add_source(Environment::with_prefix("AUTH_LGRB"))
            .build()?;

        let app_config: AppConfig = config.try_deserialize()?;

        if app_config.jwt_secret.is_empty() {
            return Err(ConfigError::Message("JWT_SECRET must be set and not empty".to_string()));
        }

        if app_config.cookie_domain.is_empty() {
            return Err(ConfigError::Message(
                "COOKIE_DOMAIN must be set and not empty".to_string(),
            ));
        }

        if app_config.database_url.is_empty() {
            return Err(ConfigError::Message(
                "DATABASE_URL must be set and not empty".to_string(),
            ));
        }

        Ok(app_config)
    }
}

static CONFIG: OnceLock<AppConfig> = OnceLock::new();

pub fn get_config() -> &'static AppConfig {
    CONFIG.get_or_init(|| AppConfig::from_env().expect("Failed to load configuration"))
}

pub static JWT_SECRET: LazyLock<SecretBox<String>> =
    LazyLock::new(|| SecretBox::new(Box::from(get_config().jwt_secret.clone())));

pub static COOKIE_DOMAIN: LazyLock<String> = LazyLock::new(|| get_config().cookie_domain.clone());

pub static DATABASE_URL: LazyLock<SecretBox<String>> =
    LazyLock::new(|| SecretBox::new(Box::from(get_config().database_url.clone())));

pub static REDIS_HOST_NAME: LazyLock<String> = LazyLock::new(|| get_config().redis_host_name.clone());

pub static CORS_ALLOWED_ORIGINS: LazyLock<String> = LazyLock::new(|| get_config().cors_allowed_origins.clone());

pub static CAPTCHA_SITE_KEY: LazyLock<String> = LazyLock::new(|| get_config().captcha_site_key.clone());

pub static CAPTCHA_SECRET_KEY: LazyLock<String> = LazyLock::new(|| get_config().captcha_secret_key.clone());

pub static TOKEN_TTL_SECONDS: LazyLock<i64> = LazyLock::new(|| get_config().token_ttl_seconds);

pub static REFRESH_TOKEN_TTL_SECONDS: LazyLock<i64> = LazyLock::new(|| get_config().refresh_token_ttl_seconds);
