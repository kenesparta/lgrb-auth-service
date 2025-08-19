use crate::domain::{
    Email, Password, User,
    data_stores::{UserStore, UserStoreError},
};
use argon2::{
    Algorithm, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier, Version,
    password_hash::SaltString, password_hash::rand_core::OsRng,
};
use sqlx::PgPool;
use std::error::Error;

pub struct PostgresUserStore {
    pool: PgPool,
}

impl PostgresUserStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl UserStore for PostgresUserStore {
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        let password_hash = compute_password_hash(user.password().as_ref().to_string())
            .await
            .map_err(|_| UserStoreError::UnexpectedError)?;

        let result = sqlx::query!(
            r#"INSERT INTO users (email, password_hash, requires_2fa) VALUES ($1, $2, $3)"#,
            user.email().as_ref(),
            password_hash,
            user.requires_2fa()
        )
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => Ok(()),
            Err(sqlx::Error::Database(db_err)) => {
                if let Some(constraint) = db_err.constraint() {
                    if constraint.contains("email") || constraint.contains("users_email") {
                        return Err(UserStoreError::UserAlreadyExists);
                    }
                }

                if db_err.code() == Some(std::borrow::Cow::Borrowed("23505")) {
                    return Err(UserStoreError::UserAlreadyExists);
                }

                Err(UserStoreError::UnexpectedError)
            }
            Err(_) => Err(UserStoreError::UnexpectedError),
        }
    }

    async fn get_user(&self, email: &Email) -> Result<User, UserStoreError> {
        let result = sqlx::query!(
            r#"SELECT email, password_hash, requires_2fa FROM users WHERE email = $1"#,
            email.as_ref()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| UserStoreError::UnexpectedError)?;

        match result {
            Some(record) => {
                let user = User::new(record.email, record.password_hash, record.requires_2fa)
                    .map_err(|_| UserStoreError::UnexpectedError)?;
                Ok(user)
            }
            None => Err(UserStoreError::UserNotFound),
        }
    }

    async fn validate_user(
        &self,
        email: &Email,
        password: &Password,
    ) -> Result<(), UserStoreError> {
        let result = sqlx::query!(
            r#"SELECT password_hash FROM users WHERE email = $1"#,
            email.as_ref()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| UserStoreError::UnexpectedError)?;

        match result {
            Some(record) => {
                verify_password_hash(record.password_hash, password.as_ref().to_string())
                    .await
                    .map_err(|_| UserStoreError::IncorrectCredentials)?;
                Ok(())
            }
            None => Err(UserStoreError::UserNotFound),
        }
    }

    async fn delete_account(&mut self, email: &Email) -> Result<(), UserStoreError> {
        let result = sqlx::query!(r#"DELETE FROM users WHERE email = $1"#, email.as_ref())
            .execute(&self.pool)
            .await
            .map_err(|_| UserStoreError::UnexpectedError)?;

        if result.rows_affected() == 0 {
            Err(UserStoreError::UserNotFound)
        } else {
            Ok(())
        }
    }
}

async fn verify_password_hash(
    expected_password_hash: String,
    password_candidate: String,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    tokio::task::spawn_blocking(move || {
        let expected_password_hash: PasswordHash<'_> = PasswordHash::new(&expected_password_hash)?;

        Argon2::default()
            .verify_password(password_candidate.as_bytes(), &expected_password_hash)
            .map_err(|e| -> Box<dyn Error + Send + Sync> { Box::new(e) })
    })
    .await?
}

async fn compute_password_hash(password: String) -> Result<String, Box<dyn Error + Send + Sync>> {
    tokio::task::spawn_blocking(move || {
        let salt: SaltString = SaltString::generate(&mut OsRng);
        let password_hash = Argon2::new(
            Algorithm::Argon2id,
            Version::V0x13,
            Params::new(15000, 2, 1, None)?,
        )
        .hash_password(password.as_bytes(), &salt)?
        .to_string();

        Ok(password_hash)
    })
    .await?
}
