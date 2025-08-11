#[cfg(test)]
use mockall::automock;

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
