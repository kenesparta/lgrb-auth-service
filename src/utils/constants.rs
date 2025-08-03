pub const JWT_COOKIE_NAME: &str = "jwt";
pub const TOKEN_TTL_SECONDS: i64 = 600;
pub const JWT_SECRET: &str = "secret";
pub mod env {
    pub const JWT_SECRET_ENV_VAR: &str = "JWT_SECRET";
}
