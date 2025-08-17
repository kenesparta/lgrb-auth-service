use crate::domain::data_stores::{UserStore, UserStoreError};
use crate::domain::{Email, Password, User};
use std::collections::HashMap;
use std::collections::hash_map::Entry;

#[derive(Default)]
pub struct HashmapUserStore {
    users: HashMap<Email, User>,
}

#[async_trait::async_trait]
impl UserStore for HashmapUserStore {
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        match self.users.entry(user.email().to_owned()) {
            Entry::Occupied(_) => Err(UserStoreError::UserAlreadyExists),
            Entry::Vacant(entry) => {
                entry.insert(user);
                Ok(())
            }
        }
    }

    async fn get_user(&self, email: &Email) -> Result<&User, UserStoreError> {
        self.users.get(email).ok_or(UserStoreError::UserNotFound)
    }

    async fn validate_user(
        &self,
        email: &Email,
        password: &Password,
    ) -> Result<(), UserStoreError> {
        let user = self.get_user(email).await?;
        if user.password() != password {
            return Err(UserStoreError::IncorrectCredentials);
        }

        Ok(())
    }

    async fn delete_account(&mut self, email: &Email) -> Result<(), UserStoreError> {
        self.users.remove(email);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fake::Fake;
    use fake::faker::internet::en::{Password as FakePassword, SafeEmail};

    #[tokio::test]
    async fn test_user_add() {
        let mut hash_map_user = HashmapUserStore::default();

        let fake_email: String = SafeEmail().fake();
        let fake_password: String = FakePassword(8..20).fake();

        let user_result = User::new(fake_email, fake_password, true);
        assert!(user_result.is_ok());

        let user_01 = user_result.unwrap();
        let result = hash_map_user.add_user(user_01).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_user_already_exist_error() {
        let mut hash_map_user = HashmapUserStore::default();
        let shared_email: String = SafeEmail().fake();
        let user_01 = User::new(shared_email.clone(), FakePassword(8..20).fake(), true).unwrap();
        let user_02 = User::new(shared_email, FakePassword(8..20).fake(), false).unwrap();
        let result1 = hash_map_user.add_user(user_01).await;
        assert!(result1.is_ok());

        let result = hash_map_user.add_user(user_02).await;
        assert_eq!(result, Err(UserStoreError::UserAlreadyExists));
    }

    #[tokio::test]
    async fn test_get_user() {
        let mut hash_map_user = HashmapUserStore::default();
        let user_01 = User::new(SafeEmail().fake(), FakePassword(8..20).fake(), true).unwrap();
        let user_02_shared: String = SafeEmail().fake();
        let user_02 = User::new(user_02_shared.clone(), FakePassword(8..20).fake(), false).unwrap();
        let user_03 = User::new(SafeEmail().fake(), FakePassword(8..20).fake(), false).unwrap();

        let result_1 = hash_map_user.add_user(user_01).await;
        assert!(result_1.is_ok());

        let result_2 = hash_map_user.add_user(user_02.clone()).await;
        assert!(result_2.is_ok());

        let result_3 = hash_map_user.add_user(user_03).await;
        assert!(result_3.is_ok());

        let user_found = hash_map_user
            .get_user(&Email::new(user_02_shared).unwrap())
            .await;
        assert_eq!(user_found, Ok(&user_02));

        let user_not_found = hash_map_user
            .get_user(&Email::new(SafeEmail().fake()).unwrap())
            .await;
        assert_eq!(user_not_found, Err(UserStoreError::UserNotFound));
    }

    #[tokio::test]
    async fn test_validate_user() {
        let mut hash_map_user = HashmapUserStore::default();

        let user_01_email_shared: String = SafeEmail().fake();
        let user_01_password_shared: String = FakePassword(8..20).fake();
        let user_01 = User::new(
            user_01_email_shared.clone(),
            user_01_password_shared.clone(),
            true,
        )
        .unwrap();

        let result_1 = hash_map_user.add_user(user_01).await;
        assert!(result_1.is_ok());

        let validation_failed = hash_map_user
            .validate_user(
                &Email::new(SafeEmail().fake()).unwrap(),
                &Password::new(FakePassword(8..20).fake()).unwrap(),
            )
            .await;
        assert_eq!(validation_failed, Err(UserStoreError::UserNotFound));

        let validation_failed = hash_map_user
            .validate_user(
                &Email::new(user_01_email_shared.clone()).unwrap(),
                &Password::new(FakePassword(8..20).fake()).unwrap(),
            )
            .await;
        assert_eq!(validation_failed, Err(UserStoreError::IncorrectCredentials));

        let validation_ok = hash_map_user
            .validate_user(
                &Email::new(user_01_email_shared).unwrap(),
                &Password::new(user_01_password_shared).unwrap(),
            )
            .await;
        assert!(validation_ok.is_ok());
    }

    #[tokio::test]
    async fn test_delete_account() {
        let mut hash_map_user = HashmapUserStore::default();

        let user_email: String = SafeEmail().fake();

        let user_01 = User::new(user_email.clone(), FakePassword(8..20).fake(), true).unwrap();
        let result_1 = hash_map_user.add_user(user_01).await;
        assert!(result_1.is_ok());

        let result_2 = hash_map_user
            .delete_account(&Email::new(user_email).unwrap())
            .await;
        assert!(result_2.is_ok());
    }
}
