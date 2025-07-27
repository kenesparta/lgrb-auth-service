use auth_service::app_state::{AppState, UserStoreType};
use auth_service::services::HashmapUserStore;
use auth_service::Application;
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() {
    let user_store: UserStoreType = Arc::new(RwLock::new(HashmapUserStore::default()));
    let app_state = AppState::new(user_store);
    let app = Application::build(app_state, "0.0.0.0:3000")
        .await
        .expect("Failed to build app");

    app.run().await.expect("Failed to run app");
}
