use crate::domain::{
    Email, LoginAttemptId, TwoFACode,
    data_stores::{TwoFACodeStore, TwoFACodeStoreError},
};
use crate::utils::TOKEN_TTL_SECONDS;
use crate::utils::redis_env::TWO_FA_CODE_PREFIX;
use redis::aio::MultiplexedConnection;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct TwoFATuple(pub String, pub String);

pub struct RedisTwoFACodeStore {
    conn: MultiplexedConnection,
}

impl RedisTwoFACodeStore {
    pub fn new(conn: MultiplexedConnection) -> Self {
        Self { conn }
    }
}

#[async_trait::async_trait]
impl TwoFACodeStore for RedisTwoFACodeStore {
    async fn add_code(
        &mut self,
        email: &Email,
        login_attempt_id: LoginAttemptId,
        code: TwoFACode,
    ) -> Result<(), TwoFACodeStoreError> {
        let two_fa = serde_json::to_string(&TwoFATuple(
            login_attempt_id.as_ref().to_string(),
            code.as_ref().to_string(),
        ))
        .map_err(|_| TwoFACodeStoreError::UnexpectedError)?;

        Ok(redis::cmd("SETEX")
            .arg(&get_key(email))
            .arg(TOKEN_TTL_SECONDS)
            .arg(&two_fa)
            .query_async::<_, ()>(&mut self.conn.clone())
            .await
            .map_err(|_| TwoFACodeStoreError::UnexpectedError)?)
    }

    async fn remove_code(&mut self, email: &Email) -> Result<(), TwoFACodeStoreError> {
        // TODO:
        // 1. Create a new key using the get_key helper function.
        // 2. Call the del command on the Redis connection to delete the 2FA code entry.
        // Return TwoFACodeStoreError::UnexpectedError if the operation fails.

        todo!()
    }

    async fn get_code(
        &self,
        email: &Email,
    ) -> Result<(LoginAttemptId, TwoFACode), TwoFACodeStoreError> {
        // TODO:
        // 1. Create a new key using the get_key helper function.
        // 2. Call the get command on the Redis connection to get the value stored for the key.
        // Return TwoFACodeStoreError::LoginAttemptIdNotFound if the operation fails.
        // If the operation succeeds, call serde_json::from_str to parse the JSON string into a TwoFATuple.
        // Then, parse the login attempt ID string and 2FA code string into a LoginAttemptId and TwoFACode type respectively.
        // Return TwoFACodeStoreError::UnexpectedError if parsing fails.

        todo!()
    }
}

fn get_key(email: &Email) -> String {
    format!("{}{}", TWO_FA_CODE_PREFIX, email.as_ref())
}
