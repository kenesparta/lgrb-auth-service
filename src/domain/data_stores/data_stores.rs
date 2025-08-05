use crate::domain::{Email, Password, User};
#[cfg(test)]
use mockall::automock;

#[derive(Debug, PartialEq)]
pub enum UserStoreError {
    UserAlreadyExists,
    UserNotFound,
    IncorrectCredentials,
    UnexpectedError,
}

#[cfg_attr(test, automock)]
#[async_trait::async_trait]
pub trait UserStore: Send + Sync {
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError>;
    async fn get_user<'a>(&'a self, email: &Email) -> Result<&'a User, UserStoreError>;
    async fn validate_user(&self, email: &Email, password: &Password)
        -> Result<(), UserStoreError>;
    async fn delete_account(&mut self, email: &Email) -> Result<(), UserStoreError>;
}

#[derive(Debug, PartialEq)]
pub enum BannedTokenStoreError {
    TokenAlreadyBanned,
    UnexpectedError,
}

#[cfg_attr(test, automock)]
#[async_trait::async_trait]
pub trait BannedTokenStore: Send + Sync {
    async fn store_token(&mut self, token: &str) -> Result<(), BannedTokenStoreError>;
    async fn is_banned(&self, token: &str) -> Result<bool, BannedTokenStoreError>;
}
