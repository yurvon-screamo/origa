use crate::domain::{OrigaError, User};
use ulid::Ulid;

#[async_trait::async_trait]
pub trait UserRepository: Send + Sync {
    async fn list(&self) -> Result<Vec<User>, OrigaError>;
    async fn find_by_id(&self, user_id: Ulid) -> Result<Option<User>, OrigaError>;
    async fn save(&self, user: &User) -> Result<(), OrigaError>;
    async fn delete(&self, user_id: Ulid) -> Result<(), OrigaError>;
}
