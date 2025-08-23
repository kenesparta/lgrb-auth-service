use crate::domain::data_stores::{BannedTokenStore, BannedTokenStoreError};
use crate::utils::TOKEN_TTL_SECONDS;
use crate::utils::redis_env::BANNED_TOKEN_KEY_PREFIX;
use redis::aio::MultiplexedConnection;

pub struct RedisBannedTokenStore {
    conn: MultiplexedConnection,
}

impl RedisBannedTokenStore {
    pub fn new(conn: MultiplexedConnection) -> Self {
        Self { conn }
    }
}

#[async_trait::async_trait]
impl BannedTokenStore for RedisBannedTokenStore {
    async fn store_token(&mut self, token: &str) -> Result<(), BannedTokenStoreError> {
        Ok(redis::cmd("SETEX")
            .arg(&get_key(token))
            .arg(*TOKEN_TTL_SECONDS)
            .arg(true)
            .query_async::<_, ()>(&mut self.conn.clone())
            .await
            .map_err(|_| BannedTokenStoreError::UnexpectedError)?)
    }

    async fn is_banned(&self, token: &str) -> Result<bool, BannedTokenStoreError> {
        redis::cmd("EXISTS")
            .arg(&get_key(token))
            .query_async(&mut self.conn.clone())
            .await
            .map_err(|_| BannedTokenStoreError::UnexpectedError)
    }
}

fn get_key(token: &str) -> String {
    format!("{}{}", BANNED_TOKEN_KEY_PREFIX, token)
}
