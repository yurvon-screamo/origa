use crate::domain::{NativeLanguage, OrigaError};
use crate::traits::UserRepository;
use tracing::{debug, info};
use ulid::Ulid;

#[derive(Clone)]
pub struct UpdateUserProfileUseCase<'a, R: UserRepository> {
    repository: &'a R,
}

impl<'a, R: UserRepository> UpdateUserProfileUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        user_id: Ulid,
        native_language: NativeLanguage,
        telegram_user_id: Option<u64>,
    ) -> Result<(), OrigaError> {
        debug!(user_id = %user_id, "Updating user profile");

        let mut user = self
            .repository
            .find_by_id(user_id)
            .await?
            .ok_or(OrigaError::UserNotFound { user_id })?;

        user.set_native_language(native_language);
        user.set_telegram_user_id(telegram_user_id);

        self.repository.save(&user).await?;

        info!(user_id = %user_id, "User profile updated");
        Ok(())
    }
}
