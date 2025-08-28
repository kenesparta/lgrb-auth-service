use crate::domain::{
    Email, Password, User,
    data_stores::{UserStore, UserStoreError},
};
use argon2::{
    Algorithm, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier, Version, password_hash::SaltString,
    password_hash::rand_core::OsRng,
};
use color_eyre::eyre::{Context, Result};
use secrecy::{ExposeSecret, SecretBox};
use sqlx::PgPool;

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
    #[tracing::instrument(name = "Adding user to PostgreSQL", skip_all)]
    async fn add_user(
        &mut self,
        user: User,
    ) -> Result<(), UserStoreError> {
        let password_hash = compute_password_hash(user.password().as_ref().expose_secret().to_string())
            .await
            .map_err(|e| UserStoreError::UnexpectedError(e))?;

        let result = sqlx::query!(
            r#"INSERT INTO users (email, password_hash, requires_2fa) VALUES ($1, $2, $3)"#,
            user.email().as_ref().expose_secret(),
            &password_hash.expose_secret(),
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

                Err(UserStoreError::UnexpectedError(db_err.into()))
            }
            Err(e) => Err(UserStoreError::UnexpectedError(e.into())),
        }
    }

    #[tracing::instrument(name = "Retrieving user from PostgreSQL", skip_all)]
    async fn get_user(
        &self,
        email: &Email,
    ) -> Result<User, UserStoreError> {
        let result = sqlx::query!(
            r#"SELECT email, password_hash, requires_2fa FROM users WHERE email = $1"#,
            email.as_ref().expose_secret()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| UserStoreError::UnexpectedError(e.into()))?;

        match result {
            Some(record) => {
                let user = User::new(record.email, record.password_hash, record.requires_2fa)
                    .map_err(|e| UserStoreError::UnexpectedError(e.into()))?;
                Ok(user)
            }
            None => Err(UserStoreError::UserNotFound),
        }
    }

    #[tracing::instrument(name = "Validating user credentials in PostgreSQL", skip_all)]
    async fn validate_user(
        &self,
        email: &Email,
        password: &Password,
    ) -> Result<(), UserStoreError> {
        let result = sqlx::query!(
            r#"SELECT password_hash FROM users WHERE email = $1"#,
            email.as_ref().expose_secret()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| UserStoreError::UnexpectedError(e.into()))?;

        match result {
            Some(record) => {
                verify_password_hash(record.password_hash, password.as_ref().expose_secret().to_string())
                    .await
                    .map_err(|_| UserStoreError::IncorrectCredentials)?;
                Ok(())
            }
            None => Err(UserStoreError::UserNotFound),
        }
    }

    #[tracing::instrument(name = "Deleting user in PostgreSQL", skip_all)]
    async fn delete_account(
        &mut self,
        email: &Email,
    ) -> Result<(), UserStoreError> {
        let result = sqlx::query!(r#"DELETE FROM users WHERE email = $1"#, email.as_ref().expose_secret())
            .execute(&self.pool)
            .await
            .map_err(|e| UserStoreError::UnexpectedError(e.into()))?;

        if result.rows_affected() == 0 {
            Err(UserStoreError::UserNotFound)
        } else {
            Ok(())
        }
    }
}

#[tracing::instrument(name = "Verify the password hash", skip_all)]
async fn verify_password_hash(
    expected_password_hash: String,
    password_candidate: String,
) -> Result<()> {
    let current_span: tracing::Span = tracing::Span::current();
    let result = tokio::task::spawn_blocking(move || {
        current_span.in_scope(|| {
            let expected_password_hash: PasswordHash<'_> = PasswordHash::new(&expected_password_hash)?;

            Argon2::default()
                .verify_password(password_candidate.as_bytes(), &expected_password_hash)
                .wrap_err("failed to verify the password hash")
        })
    })
    .await;

    result?
}

#[tracing::instrument(name = "Computing password hash", skip_all)]
async fn compute_password_hash(password: String) -> Result<SecretBox<String>> {
    let current_span: tracing::Span = tracing::Span::current();

    let result = tokio::task::spawn_blocking(move || {
        current_span.in_scope(|| {
            let salt: SaltString = SaltString::generate(&mut OsRng);
            let password_hash = Argon2::new(Algorithm::Argon2id, Version::V0x13, Params::new(15000, 2, 1, None)?)
                .hash_password(password.as_bytes(), &salt)?
                .to_string();

            Ok(SecretBox::new(Box::from(password_hash)))
        })
    })
    .await;

    result?
}
