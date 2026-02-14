use crate::domain::{OrigaError, User};
use std::future::Future;
use ulid::Ulid;

pub trait UserRepository {
    fn list(&self) -> impl Future<Output = Result<Vec<User>, OrigaError>>;
    fn find_by_id(&self, user_id: Ulid) -> impl Future<Output = Result<Option<User>, OrigaError>>;
    fn find_by_telegram_id(
        &self,
        telegram_id: &u64,
    ) -> impl Future<Output = Result<Option<User>, OrigaError>>;
    fn save(&self, user: &User) -> impl Future<Output = Result<(), OrigaError>>;
    fn delete(&self, user_id: Ulid) -> impl Future<Output = Result<(), OrigaError>>;
}
