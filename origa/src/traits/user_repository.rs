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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{NativeLanguage, User};
    use std::cell::Cell;
    use std::future::Future;

    struct RecordingRepository {
        save_calls: Cell<usize>,
        sync_calls: Cell<usize>,
    }

    impl UserRepository for RecordingRepository {
        fn get_current_user(&self) -> impl Future<Output = Result<Option<User>, OrigaError>> {
            std::future::ready(Ok(None))
        }

        fn save(&self, _user: &User) -> impl Future<Output = Result<(), OrigaError>> {
            self.save_calls.set(self.save_calls.get() + 1);
            std::future::ready(Ok(()))
        }

        fn save_sync(&self, user: &User) -> impl Future<Output = Result<(), OrigaError>> {
            let sync_calls = &self.sync_calls;
            async move {
                sync_calls.set(sync_calls.get() + 1);
                self.save(user).await
            }
        }

        fn delete(&self, _user_id: Ulid) -> impl Future<Output = Result<(), OrigaError>> {
            std::future::ready(Ok(()))
        }
    }

    #[tokio::test]
    async fn save_sync_defaults_to_save_for_implementations() {
        // Arrange — a fake without the save_sync override, so the trait
        // default is exercised.
        struct DefaultOnlyRepository {
            save_calls: Cell<usize>,
        }

        impl UserRepository for DefaultOnlyRepository {
            fn get_current_user(&self) -> impl Future<Output = Result<Option<User>, OrigaError>> {
                std::future::ready(Ok(None))
            }

            fn save(&self, _user: &User) -> impl Future<Output = Result<(), OrigaError>> {
                self.save_calls.set(self.save_calls.get() + 1);
                std::future::ready(Ok(()))
            }

            fn delete(&self, _user_id: Ulid) -> impl Future<Output = Result<(), OrigaError>> {
                std::future::ready(Ok(()))
            }
        }

        let user = User::new(
            "save-sync@example.com".to_string(),
            NativeLanguage::Russian,
            None,
        );
        let repo = DefaultOnlyRepository {
            save_calls: Cell::new(0),
        };

        // Act
        repo.save_sync(&user).await.expect("default delegates");

        // Assert — the default routed through save.
        assert_eq!(repo.save_calls.get(), 1);
    }

    #[tokio::test]
    async fn save_sync_override_replaces_the_default() {
        // Arrange
        let user = User::new(
            "override@example.com".to_string(),
            NativeLanguage::Russian,
            None,
        );
        let repo = RecordingRepository {
            save_calls: Cell::new(0),
            sync_calls: Cell::new(0),
        };

        // Act
        repo.save_sync(&user).await.expect("override runs");

        // Assert — the override ran instead of the default save path.
        assert_eq!(repo.sync_calls.get(), 1);
        assert_eq!(repo.save_calls.get(), 1, "override chose to delegate");
    }
}
