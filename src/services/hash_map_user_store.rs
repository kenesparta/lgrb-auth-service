use crate::domain::data_stores::{UserStore, UserStoreError};
use crate::domain::User;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

#[derive(Default)]
pub struct HashmapUserStore {
    users: HashMap<String, User>,
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

    async fn get_user(&self, email: &str) -> Result<&User, UserStoreError> {
        self.users.get(email).ok_or(UserStoreError::UserNotFound)
    }

    async fn validate_user(&self, email: &str, password: &str) -> Result<(), UserStoreError> {
        let user = self.get_user(email).await?;
        if user.password() != password {
            return Err(UserStoreError::InvalidCredentials);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_user_add() {
        let mut hash_map_user = HashmapUserStore {
            users: HashMap::new(),
        };
        let user_01 = User::new(
            "b3d59bed-fb09-45ab-afe4-19dc36eb7b0c@example.com".to_owned(),
            "YMchRXmRYU7KId62".to_owned(),
            true,
        );
        let result = hash_map_user.add_user(user_01).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_user_already_exist_error() {
        let mut hash_map_user = HashmapUserStore {
            users: HashMap::new(),
        };
        let user_01 = User::new(
            "973d5517-a64a-425c-9a9c-4edea6727999@example.com".to_owned(),
            "lkVXC19llW7+il0Q".to_owned(),
            true,
        );
        let user_02 = User::new(
            "973d5517-a64a-425c-9a9c-4edea6727999@example.com".to_owned(),
            "3F14XASpMQ9Tw2iV".to_owned(),
            false,
        );
        let result1 = hash_map_user.add_user(user_01).await;
        assert!(result1.is_ok());

        let result = hash_map_user.add_user(user_02).await;
        assert_eq!(result, Err(UserStoreError::UserAlreadyExists));
    }

    #[tokio::test]
    async fn test_get_user() {
        let mut hash_map_user = HashmapUserStore {
            users: HashMap::new(),
        };
        let user_01 = User::new(
            "a20118e3-4063-49d4-be53-2b6bd9a8fc3c@example.com".to_owned(),
            "u2O5+zGqn+KxzBI4".to_owned(),
            true,
        );
        let user_02 = User::new(
            "3e5f8404-ce14-4bcc-aaee-e7201ea6bf18@example.com".to_owned(),
            "ps5ZbKwSI4VhQ6Ti".to_owned(),
            false,
        );
        let user_03 = User::new(
            "22212582-7517-4a68-9072-780d05ce508a@example.com".to_owned(),
            "nmTsz8WgA9stMWcR".to_owned(),
            false,
        );

        let result_1 = hash_map_user.add_user(user_01).await;
        assert!(result_1.is_ok());

        let result_2 = hash_map_user.add_user(user_02.clone()).await;
        assert!(result_2.is_ok());

        let result_3 = hash_map_user.add_user(user_03).await;
        assert!(result_3.is_ok());

        let user_found = hash_map_user
            .get_user("3e5f8404-ce14-4bcc-aaee-e7201ea6bf18@example.com")
            .await;
        assert_eq!(user_found, Ok(&user_02));

        let user_not_found = hash_map_user
            .get_user("99409174-5b16-4a0d-be9f-9e6bb62e840c@example.com")
            .await;
        assert_eq!(user_not_found, Err(UserStoreError::UserNotFound));
    }

    #[tokio::test]
    async fn test_validate_user() {
        let mut hash_map_user = HashmapUserStore {
            users: HashMap::new(),
        };
        let user_01 = User::new(
            "7cdbaaa9-1e78-4682-8294-0303edeb49bb@example.com".to_owned(),
            "WIDXR83rXJuxTuGY".to_owned(),
            true,
        );

        let result_1 = hash_map_user.add_user(user_01).await;
        assert!(result_1.is_ok());

        let validation_failed = hash_map_user
            .validate_user(
                "ad0ec61e-2273-4e16-9170-266261d22d87@example.com",
                "eDAyfl3yWjaky9S+",
            )
            .await;
        assert_eq!(validation_failed, Err(UserStoreError::UserNotFound));

        let validation_failed = hash_map_user
            .validate_user(
                "7cdbaaa9-1e78-4682-8294-0303edeb49bb@example.com",
                "j6Vl9u4i1dECShDs",
            )
            .await;
        assert_eq!(validation_failed, Err(UserStoreError::InvalidCredentials));

        let validation_ok = hash_map_user
            .validate_user(
                "7cdbaaa9-1e78-4682-8294-0303edeb49bb@example.com",
                "WIDXR83rXJuxTuGY",
            )
            .await;
        assert!(validation_ok.is_ok());
    }
}
