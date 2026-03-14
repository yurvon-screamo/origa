use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use ulid::Ulid;

use crate::domain::{OrigaError, User};

#[derive(Clone)]
pub struct InMemoryUserRepository {
    users: Arc<Mutex<HashMap<Ulid, User>>>,
}

impl Default for InMemoryUserRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryUserRepository {
    pub fn new() -> Self {
        Self {
            users: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn with_user(user: User) -> Self {
        let repo = Self::new();
        let id = user.id();
        repo.users.lock().unwrap().insert(id, user);
        repo
    }
}

impl crate::traits::UserRepository for InMemoryUserRepository {
    async fn get_current_user(&self) -> Result<Option<User>, OrigaError> {
        Ok(self.users.lock().unwrap().values().next().cloned())
    }

    async fn save(&self, user: &User) -> Result<(), OrigaError> {
        self.users.lock().unwrap().insert(user.id(), user.clone());
        Ok(())
    }

    async fn save_sync(&self, user: &User) -> Result<(), OrigaError> {
        self.save(user).await
    }

    async fn delete(&self, user_id: Ulid) -> Result<(), OrigaError> {
        self.users.lock().unwrap().remove(&user_id);
        Ok(())
    }
}
