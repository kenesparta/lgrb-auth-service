pub const JWT_COOKIE_NAME: &str = "jwt";
pub const JWT_REFRESH_COOKIE_NAME: &str = "jwt-refresh";
pub const PGSQL_MAX_CONNECTIONS: u32 = 10;

pub mod prod {
    pub const APP_ADDRESS: &str = "0.0.0.0:3000";
}

pub mod test {
    pub const APP_ADDRESS: &str = "127.0.0.1:0";
}

pub mod email {
    pub const SUBJECT: &str = "Let's get Rusty Bootcamp code";
}

pub mod redis_env {
    pub const BANNED_TOKEN_KEY_PREFIX: &str = "banned_token:";
    pub const TWO_FA_CODE_PREFIX: &str = "two_fa_code:";
}
