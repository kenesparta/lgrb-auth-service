use crate::utils::env;
use dotenvy::dotenv;
use lazy_static::lazy_static;
use std::env as std_env;

lazy_static! {
    pub static ref JWT_SECRET: String = set_token();
    pub static ref COOKIE_SUBDOMAIN: String = set_cookie_subdomain();
}

fn set_token() -> String {
    dotenv().ok();

    let secret = std_env::var(env::JWT_SECRET_ENV_VAR).expect("JWT_SECRET must be set.");
    if secret.is_empty() {
        panic!("JWT_SECRET must not be empty.");
    }

    secret
}

fn set_cookie_subdomain() -> String {
    dotenv().ok();

    let cookie_subdomain =
        std_env::var(env::COOKIE_SUBDOMAIN_ENV_VAR).expect("COOKIE_SUBDOMAIN must be set.");
    if cookie_subdomain.is_empty() {
        panic!("COOKIE_SUBDOMAIN must not be empty.");
    }

    cookie_subdomain
}
