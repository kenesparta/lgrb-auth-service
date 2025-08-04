pub const JWT_COOKIE_NAME: &str = "jwt";
pub const TOKEN_TTL_SECONDS: i64 = 600;
pub const JWT_SECRET: &str = "secret";
pub mod env {
    pub const JWT_SECRET_ENV_VAR: &str = "JWT_SECRET";
}

pub mod prod {
    pub const APP_ADDRESS: &str = "0.0.0.0:3000";
}

pub mod test {
    pub const APP_ADDRESS: &str = "127.0.0.1:0";
}
