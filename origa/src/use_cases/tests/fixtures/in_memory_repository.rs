use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use ulid::Ulid;

use crate::domain::{OrigaError, User};

#[derive(Clone)]
pub struct InMemoryUserRepository {
    users: Arc<Mutex<HashMap<Ulid, User>>>,
    current_user_id: Arc<Mutex<Option<Ulid>>>,
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
            current_user_id: Arc::new(Mutex::new(None)),
        }
    }

    pub fn with_user(user: User) -> Self {
        let id = user.id();
        let repo = Self::new();
        repo.users.lock().unwrap().insert(id, user);
        *repo.current_user_id.lock().unwrap() = Some(id);
        repo
    }
}

impl crate::traits::UserRepository for InMemoryUserRepository {
    async fn get_current_user(&self) -> Result<Option<User>, OrigaError> {
        let users = self.users.lock().unwrap();
        let current_id = self.current_user_id.lock().unwrap();
        Ok(current_id.and_then(|id| users.get(&id).cloned()))
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
