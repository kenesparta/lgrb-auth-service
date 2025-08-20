use crate::domain::data_stores::{TwoFACodeStore, TwoFACodeStoreError};
use crate::domain::{Email, LoginAttemptId, TwoFACode};
use async_trait::async_trait;
use std::collections::HashMap;
use std::collections::hash_map::Entry;

#[derive(Default)]
pub struct HashmapTwoFACodeStore {
    codes: HashMap<Email, (LoginAttemptId, TwoFACode)>,
}

#[async_trait]
impl TwoFACodeStore for HashmapTwoFACodeStore {
    async fn add_code(
        &mut self,
        email: &Email,
        login_attempt_id: LoginAttemptId,
        code: TwoFACode,
    ) -> Result<(), TwoFACodeStoreError> {
        match self.codes.entry(email.to_owned()) {
            Entry::Occupied(mut entry) => {
                entry.insert((login_attempt_id, code));
            }
            Entry::Vacant(entry) => {
                entry.insert((login_attempt_id, code));
            }
        }
        Ok(())
    }

    async fn remove_code(&mut self, email: &Email) -> Result<(), TwoFACodeStoreError> {
        self.codes.remove(email);
        Ok(())
    }

    async fn get_code(
        &self,
        email: &Email,
    ) -> Result<(LoginAttemptId, TwoFACode), TwoFACodeStoreError> {
        self.codes
            .get(email)
            .cloned()
            .ok_or(TwoFACodeStoreError::UserNotFound)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fake::Fake;
    use fake::faker::internet::en::SafeEmail;

    #[tokio::test]
    async fn test_add_code_new_email() {
        let mut store = HashmapTwoFACodeStore::default();
        let email = &Email::new(SafeEmail().fake()).unwrap();
        let login_attempt_id = LoginAttemptId::default();
        let code = TwoFACode::default();

        let result = store
            .add_code(email, login_attempt_id.clone(), code.clone())
            .await;

        assert!(result.is_ok());

        let stored_code = store.get_code(email).await.unwrap();
        assert_eq!(stored_code.0, login_attempt_id);
        assert_eq!(stored_code.1, code);
    }

    #[tokio::test]
    async fn test_add_code_existing_email_overwrites() {
        let mut store = HashmapTwoFACodeStore::default();
        let email = &Email::new(SafeEmail().fake()).unwrap();

        let login_attempt_id_1 = LoginAttemptId::default();
        let code_1 = TwoFACode::default();
        let result_1 = store.add_code(email, login_attempt_id_1, code_1).await;
        assert!(result_1.is_ok());

        let login_attempt_id_2 = LoginAttemptId::default();
        let code_2 = TwoFACode::default();
        let result_2 = store
            .add_code(email, login_attempt_id_2.clone(), code_2.clone())
            .await;
        assert!(result_2.is_ok());

        let stored_code = store.get_code(email).await.unwrap();
        assert_eq!(stored_code.0, login_attempt_id_2);
        assert_eq!(stored_code.1, code_2);
    }

    #[tokio::test]
    async fn test_add_code_multiple_emails() {
        let mut store = HashmapTwoFACodeStore::default();

        let email_1 = &Email::new(SafeEmail().fake()).unwrap();
        let login_attempt_id_1 = LoginAttemptId::default();
        let code_1 = TwoFACode::default();

        let email_2 = &Email::new(SafeEmail().fake()).unwrap();
        let login_attempt_id_2 = LoginAttemptId::default();
        let code_2 = TwoFACode::default();

        let result_1 = store
            .add_code(email_1, login_attempt_id_1.clone(), code_1.clone())
            .await;
        let result_2 = store
            .add_code(email_2, login_attempt_id_2.clone(), code_2.clone())
            .await;

        assert!(result_1.is_ok());
        assert!(result_2.is_ok());

        let stored_code_1 = store.get_code(email_1).await.unwrap();
        assert_eq!(stored_code_1.0, login_attempt_id_1);
        assert_eq!(stored_code_1.1, code_1);

        let stored_code_2 = store.get_code(email_2).await.unwrap();
        assert_eq!(stored_code_2.0, login_attempt_id_2);
        assert_eq!(stored_code_2.1, code_2);
    }

    #[tokio::test]
    async fn test_get_code_existing_email() {
        let mut store = HashmapTwoFACodeStore::default();
        let email = &Email::new(SafeEmail().fake()).unwrap();
        let login_attempt_id = LoginAttemptId::default();
        let code = TwoFACode::default();

        store
            .add_code(email, login_attempt_id.clone(), code.clone())
            .await
            .unwrap();

        let result = store.get_code(email).await;
        assert!(result.is_ok());

        let stored_data = result.unwrap();
        assert_eq!(stored_data.0, login_attempt_id);
        assert_eq!(stored_data.1, code);
    }

    #[tokio::test]
    async fn test_get_code_nonexistent_email() {
        let store = HashmapTwoFACodeStore::default();
        let email = &Email::new(SafeEmail().fake()).unwrap();

        let result = store.get_code(email).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), TwoFACodeStoreError::UserNotFound);
    }

    #[tokio::test]
    async fn test_remove_code_existing_email() {
        let mut store = HashmapTwoFACodeStore::default();
        let email = &Email::new(SafeEmail().fake()).unwrap();
        let login_attempt_id = LoginAttemptId::default();
        let code = TwoFACode::default();

        store.add_code(email, login_attempt_id, code).await.unwrap();

        assert!(store.get_code(&email).await.is_ok());

        let result = store.remove_code(email).await;
        assert!(result.is_ok());

        let get_result = store.get_code(email).await;
        assert!(get_result.is_err());
        assert_eq!(get_result.unwrap_err(), TwoFACodeStoreError::UserNotFound);
    }

    #[tokio::test]
    async fn test_remove_code_nonexistent_email() {
        let mut store = HashmapTwoFACodeStore::default();
        let email = Email::new(SafeEmail().fake()).unwrap();

        let result = store.remove_code(&email).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_remove_code_does_not_affect_other_emails() {
        let mut store = HashmapTwoFACodeStore::default();

        let email_1 = &Email::new(SafeEmail().fake()).unwrap();
        let login_attempt_id_1 = LoginAttemptId::default();
        let code_1 = TwoFACode::default();

        let email_2 = &Email::new(SafeEmail().fake()).unwrap();
        let login_attempt_id_2 = LoginAttemptId::default();
        let code_2 = TwoFACode::default();

        store
            .add_code(email_1, login_attempt_id_1.clone(), code_1.clone())
            .await
            .unwrap();
        store
            .add_code(email_2, login_attempt_id_2.clone(), code_2.clone())
            .await
            .unwrap();

        let result = store.remove_code(&email_1).await;
        assert!(result.is_ok());

        let get_result_1 = store.get_code(&email_1).await;
        assert!(get_result_1.is_err());
        assert_eq!(get_result_1.unwrap_err(), TwoFACodeStoreError::UserNotFound);

        let get_result_2 = store.get_code(&email_2).await;
        assert!(get_result_2.is_ok());
        let stored_data = get_result_2.unwrap();
        assert_eq!(stored_data.0, login_attempt_id_2);
        assert_eq!(stored_data.1, code_2);
    }

    #[tokio::test]
    async fn test_store_lifecycle() {
        let mut store = HashmapTwoFACodeStore::default();
        let email = &Email::new(SafeEmail().fake()).unwrap();
        let login_attempt_id = LoginAttemptId::default();
        let code = TwoFACode::default();
        assert!(store.get_code(email).await.is_err());

        store
            .add_code(email, login_attempt_id.clone(), code.clone())
            .await
            .unwrap();
        assert!(store.get_code(email).await.is_ok());

        // Remove code
        store.remove_code(email).await.unwrap();
        assert!(store.get_code(email).await.is_err());

        // Add again after removal
        store
            .add_code(email, login_attempt_id.clone(), code.clone())
            .await
            .unwrap();
        let stored_data = store.get_code(email).await.unwrap();
        assert_eq!(stored_data.0, login_attempt_id);
        assert_eq!(stored_data.1, code);
    }

    #[tokio::test]
    async fn test_default_store_is_empty() {
        let store = HashmapTwoFACodeStore::default();
        let email = Email::new(SafeEmail().fake()).unwrap();

        let result = store.get_code(&email).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), TwoFACodeStoreError::UserNotFound);
    }
}
