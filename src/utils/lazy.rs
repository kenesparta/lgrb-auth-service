use crate::utils::env;
use dotenvy::dotenv;
use lazy_static::lazy_static;
use std::env as std_env;

lazy_static! {
    pub static ref JWT_SECRET: String = set_token();
    pub static ref COOKIE_DOMAIN: String = set_cookie_domain();
    pub static ref DATABASE_URL: String = set_database_url();
}

fn set_token() -> String {
    dotenv().ok();

    let secret = std_env::var(env::JWT_SECRET_ENV_VAR).expect("JWT_SECRET must be set.");
    if secret.is_empty() {
        panic!("JWT_SECRET must not be empty.");
    }

    secret
}

fn set_cookie_domain() -> String {
    dotenv().ok();

    let cookie_subdomain =
        std_env::var(env::COOKIE_DOMAIN_ENV_VAR).expect("COOKIE_DOMAIN must be set.");
    if cookie_subdomain.is_empty() {
        panic!("COOKIE_DOMAIN must not be empty.");
    }

    cookie_subdomain
}

fn set_database_url() -> String {
    dotenv().ok();

    let set_database_url =
        std_env::var(env::DATABASE_URL_ENV_VAR).expect("DATABASE_URL must be set.");
    if set_database_url.is_empty() {
        panic!("DATABASE_URL must not be empty.");
    }

    set_database_url
}
