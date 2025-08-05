use crate::domain::data_stores::{BannedTokenStore, BannedTokenStoreError};
use async_trait::async_trait;
use std::collections::HashSet;

#[derive(Default)]
pub struct HashsetBannedTokenStore {
    tokens: HashSet<String>,
}

#[async_trait]
impl BannedTokenStore for HashsetBannedTokenStore {
    async fn store_token(&mut self, token: &str) -> Result<(), BannedTokenStoreError> {
        if !self.tokens.insert(token.to_string()) {
            return Err(BannedTokenStoreError::TokenAlreadyBanned);
        }
        Ok(())
    }

    async fn is_banned(&self, token: &str) -> Result<bool, BannedTokenStoreError> {
        Ok(self.tokens.contains(token))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fake::Fake;
    use fake::faker::lorem::en::{Sentence, Word};

    #[tokio::test]
    async fn test_store_token_success() {
        let mut store = HashsetBannedTokenStore::default();

        let fake_token: String = Word().fake();

        let result = store.store_token(&fake_token).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_store_duplicate_token_returns_error() {
        let mut store = HashsetBannedTokenStore::default();

        let fake_token: String = Word().fake();

        let result1 = store.store_token(&fake_token).await;
        assert!(result1.is_ok());

        let result2 = store.store_token(&fake_token).await;
        assert_eq!(result2, Err(BannedTokenStoreError::TokenAlreadyBanned));
    }

    #[tokio::test]
    async fn test_is_banned_returns_true_for_stored_token() {
        let mut store = HashsetBannedTokenStore::default();

        let fake_token: String = Sentence(1..3).fake();

        let store_result = store.store_token(&fake_token).await;
        assert!(store_result.is_ok());

        let is_banned_result = store.is_banned(&fake_token).await;
        assert_eq!(is_banned_result, Ok(true));
    }

    #[tokio::test]
    async fn test_is_banned_returns_false_for_non_stored_token() {
        let store = HashsetBannedTokenStore::default();

        let fake_token: String = Word().fake();

        let is_banned_result = store.is_banned(&fake_token).await;
        assert_eq!(is_banned_result, Ok(false));
    }

    #[tokio::test]
    async fn test_multiple_tokens_storage() {
        let mut store = HashsetBannedTokenStore::default();

        let token1: String = Word().fake();
        let token2: String = Word().fake();
        let token3: String = Sentence(1..2).fake();

        let result1 = store.store_token(&token1).await;
        assert!(result1.is_ok());

        let result2 = store.store_token(&token2).await;
        assert!(result2.is_ok());

        let result3 = store.store_token(&token3).await;
        assert!(result3.is_ok());

        // Verify all tokens are banned
        assert_eq!(store.is_banned(&token1).await, Ok(true));
        assert_eq!(store.is_banned(&token2).await, Ok(true));
        assert_eq!(store.is_banned(&token3).await, Ok(true));

        // Verify a non-stored token is not banned
        let non_stored_token: String = Word().fake();
        assert_eq!(store.is_banned(&non_stored_token).await, Ok(false));
    }

    #[tokio::test]
    async fn test_empty_token() {
        let mut store = HashsetBannedTokenStore::default();

        let empty_token = "";

        let store_result = store.store_token(empty_token).await;
        assert!(store_result.is_ok());

        let is_banned_result = store.is_banned(empty_token).await;
        assert_eq!(is_banned_result, Ok(true));

        let duplicate_result = store.store_token(empty_token).await;
        assert_eq!(
            duplicate_result,
            Err(BannedTokenStoreError::TokenAlreadyBanned)
        );
    }
}
