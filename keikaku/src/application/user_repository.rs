use crate::domain::{KeikakuError, User};
use ulid::Ulid;

#[async_trait::async_trait]
pub trait UserRepository: Send + Sync {
    async fn list(&self) -> Result<Vec<User>, KeikakuError>;
    async fn find_by_id(&self, user_id: Ulid) -> Result<Option<User>, KeikakuError>;
    async fn save(&self, user: &User) -> Result<(), KeikakuError>;
    async fn delete(&self, user_id: Ulid) -> Result<(), KeikakuError>;
}
