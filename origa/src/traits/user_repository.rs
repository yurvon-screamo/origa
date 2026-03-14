use crate::domain::{OrigaError, User};
use std::future::Future;
use ulid::Ulid;

pub trait UserRepository {
    fn get_current_user(&self) -> impl Future<Output = Result<Option<User>, OrigaError>>;

    fn save(&self, user: &User) -> impl Future<Output = Result<(), OrigaError>>;
    fn save_sync(&self, user: &User) -> impl Future<Output = Result<(), OrigaError>> {
        self.save(user)
    }
    fn delete(&self, user_id: Ulid) -> impl Future<Output = Result<(), OrigaError>>;
}
