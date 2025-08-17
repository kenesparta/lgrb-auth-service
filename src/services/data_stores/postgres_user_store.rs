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
        todo!()
    }

    async fn get_user<'user_store>(
        &'user_store self,
        email: &Email,
    ) -> Result<&'user_store User, UserStoreError> {
        todo!()
    }

    async fn validate_user(
        &self,
        email: &Email,
        password: &Password,
    ) -> Result<(), UserStoreError> {
        todo!()
    }

    async fn delete_account(&mut self, email: &Email) -> Result<(), UserStoreError> {
        todo!()
    }
    // TODO: Implement all required methods. Note that you will need to make SQL queries against our PostgreSQL instance inside these methods.
}

// Helper function to verify if a given password matches an expected hash
// TODO: Hashing is a CPU-intensive operation. To avoid blocking
// other async tasks, update this function to perform hashing on a
// separate thread pool using tokio::task::spawn_blocking. Note that you
// will need to update the input parameters to be String types instead of &str
async fn verify_password_hash(
    expected_password_hash: &str,
    password_candidate: &str,
) -> Result<(), Box<dyn Error>> {
    let expected_password_hash: PasswordHash<'_> = PasswordHash::new(expected_password_hash)?;

    Argon2::default()
        .verify_password(password_candidate.as_bytes(), &expected_password_hash)
        .map_err(|e| e.into())
}

// Helper function to hash passwords before persisting them in the database.
// TODO: Hashing is a CPU-intensive operation. To avoid blocking
// other async tasks, update this function to perform hashing on a
// separate thread pool using tokio::task::spawn_blocking. Note that you
// will need to update the input parameters to be String types instead of &str
async fn compute_password_hash(password: &str) -> Result<String, Box<dyn Error>> {
    let salt: SaltString = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::new(
        Algorithm::Argon2id,
        Version::V0x13,
        Params::new(15000, 2, 1, None)?,
    )
    .hash_password(password.as_bytes(), &salt)?
    .to_string();

    Ok(password_hash)
}
