use origa::domain::{OrigaError, User};
use origa::application::user_repository::UserRepository;
use async_trait::async_trait;
use ulid::Ulid;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

pub struct InMemoryUserRepository {
    users: Arc<Mutex<HashMap<Ulid, User>>>,
    username_index: Arc<Mutex<HashMap<String, Ulid>>>,
}

impl InMemoryUserRepository {
    pub fn new() -> Self {
        Self {
            users: Arc::new(Mutex::new(HashMap::new())),
            username_index: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn find_or_create_user(&self, username: String) -> Result<User, OrigaError> {
        let mut username_index = self.username_index.lock()
            .map_err(|e| OrigaError::RepositoryError { reason: format!("Lock error: {}", e) })?;

        if let Some(user_id) = username_index.get(&username) {
            if let Some(user) = self.users.lock()
                .map_err(|e| OrigaError::RepositoryError { reason: format!("Lock error: {}", e) })?
                .get(user_id)
            {
                let user: User = user.clone();
                return Ok(user);
            }
        }

        let user = User::new(
            username.clone(),
            origa::domain::JapaneseLevel::N5,
            origa::domain::NativeLanguage::Russian,
            None,
        );

        let user_id = user.id();
        
        username_index.insert(username, user_id);
        
        self.users.lock()
            .map_err(|e| OrigaError::RepositoryError { reason: format!("Lock error: {}", e) })?
            .insert(user_id, user.clone());

        Ok(user)
    }
}

#[async_trait(?Send)]
impl UserRepository for InMemoryUserRepository {
    async fn list(&self) -> Result<Vec<User>, OrigaError> {
        let users = self.users.lock()
            .map_err(|e| OrigaError::RepositoryError { reason: format!("Lock error: {}", e) })?;
        Ok(users.values().cloned().collect())
    }

    async fn find_by_id(&self, user_id: Ulid) -> Result<Option<User>, OrigaError> {
        let users = self.users.lock()
            .map_err(|e| OrigaError::RepositoryError { reason: format!("Lock error: {}", e) })?;
        Ok(users.get(&user_id).cloned())
    }

    async fn find_by_telegram_id(&self, telegram_id: &u64) -> Result<Option<User>, OrigaError> {
        let users = self.users.lock()
            .map_err(|e| OrigaError::RepositoryError { reason: format!("Lock error: {}", e) })?;
        
        for user in users.values() {
            let user_telegram_id: Option<u64> = user.telegram_user_id().copied();
            if user_telegram_id == Some(*telegram_id) {
                let user: User = user.clone();
                return Ok(Some(user));
            }
        }
        Ok(None)
    }

    async fn save(&self, user: &User) -> Result<(), OrigaError> {
        let user_id = user.id();
        let username: String = user.username().to_string();
        
        let mut username_index = self.username_index.lock()
            .map_err(|e| OrigaError::RepositoryError { reason: format!("Lock error: {}", e) })?;
        
        username_index.insert(username, user_id);
        
        let mut users = self.users.lock()
            .map_err(|e| OrigaError::RepositoryError { reason: format!("Lock error: {}", e) })?;
        
        users.insert(user_id, user.clone());
        Ok(())
    }

    async fn delete(&self, user_id: Ulid) -> Result<(), OrigaError> {
        let mut users = self.users.lock()
            .map_err(|e| OrigaError::RepositoryError { reason: format!("Lock error: {}", e) })?;
        
        if let Some(user) = users.remove(&user_id) {
            let mut username_index = self.username_index.lock()
                .map_err(|e| OrigaError::RepositoryError { reason: format!("Lock error: {}", e) })?;
            
            let username: String = user.username().to_string();
            username_index.remove(&username);
        }
        
        Ok(())
    }
}

impl Default for InMemoryUserRepository {
    fn default() -> Self {
        Self::new()
    }
}
