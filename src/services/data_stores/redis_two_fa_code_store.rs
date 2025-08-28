use crate::domain::{
    Email, LoginAttemptId, TwoFACode,
    data_stores::{TwoFACodeStore, TwoFACodeStoreError},
};
use crate::utils::TOKEN_TTL_SECONDS;
use crate::utils::redis_env::TWO_FA_CODE_PREFIX;
use redis::aio::MultiplexedConnection;
use secrecy::{ExposeSecret, SecretBox};
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
            login_attempt_id.as_ref().expose_secret().to_string(),
            code.as_ref().to_string(),
        ))
        .map_err(|e| TwoFACodeStoreError::UnexpectedError(e.into()))?;

        Ok(redis::cmd("SETEX")
            .arg(&get_key(email))
            .arg(*TOKEN_TTL_SECONDS)
            .arg(&two_fa)
            .query_async::<_, ()>(&mut self.conn.clone())
            .await
            .map_err(|e| TwoFACodeStoreError::UnexpectedError(e.into()))?)
    }

    async fn remove_code(
        &mut self,
        email: &Email,
    ) -> Result<(), TwoFACodeStoreError> {
        Ok(redis::cmd("DEL")
            .arg(&get_key(email))
            .query_async::<_, ()>(&mut self.conn.clone())
            .await
            .map_err(|e| TwoFACodeStoreError::UnexpectedError(e.into()))?)
    }

    async fn get_code(
        &self,
        email: &Email,
    ) -> Result<(LoginAttemptId, TwoFACode), TwoFACodeStoreError> {
        let key = get_key(email);

        let value = redis::cmd("GET")
            .arg(&key)
            .query_async::<_, Option<String>>(&mut self.conn.clone())
            .await
            .map_err(|e| TwoFACodeStoreError::UnexpectedError(e.into()))?;

        match value {
            Some(json_string) => {
                let two_fa_tuple: TwoFATuple =
                    serde_json::from_str(&json_string).map_err(|e| TwoFACodeStoreError::UnexpectedError(e.into()))?;

                let login_attempt_id = LoginAttemptId::new(SecretBox::new(Box::from(two_fa_tuple.0)))
                    .map_err(|e| TwoFACodeStoreError::UnexpectedError(e.into()))?;

                let two_fa_code =
                    TwoFACode::new(two_fa_tuple.1).map_err(|e| TwoFACodeStoreError::UnexpectedError(e.into()))?;

                Ok((login_attempt_id, two_fa_code))
            }
            None => Err(TwoFACodeStoreError::LoginAttemptIdNotFound),
        }
    }
}

fn get_key(email: &Email) -> String {
    format!("{}{}", TWO_FA_CODE_PREFIX, email.as_ref().expose_secret())
}
