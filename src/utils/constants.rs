pub const JWT_COOKIE_NAME: &str = "jwt";
pub const JWT_REFRESH_COOKIE_NAME: &str = "jwt-refresh";
pub const TOKEN_TTL_SECONDS: i64 = 600;
pub const REFRESH_TOKEN_TTL_SECONDS: i64 = 3600;

pub mod env {
    pub const JWT_SECRET_ENV_VAR: &str = "JWT_SECRET";
    pub const COOKIE_DOMAIN_ENV_VAR: &str = "COOKIE_DOMAIN";
}

pub mod prod {
    pub const APP_ADDRESS: &str = "0.0.0.0:3000";
}

pub mod test {
    pub const APP_ADDRESS: &str = "127.0.0.1:0";
}

pub mod email {
    pub const SUBJECT: &str = "Let's get Rusty Bootcamp code";
}
