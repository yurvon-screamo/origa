use crate::application::UserRepository;
use crate::domain::{JapaneseLevel, NativeLanguage, OrigaError};
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
        current_japanese_level: JapaneseLevel,
        native_language: NativeLanguage,
        duolingo_jwt_token: Option<String>,
        telegram_user_id: Option<u64>,
        reminders_enabled: bool,
    ) -> Result<(), OrigaError> {
        debug!(user_id = %user_id, "Updating user profile");

        let mut user = self
            .repository
            .find_by_id(user_id)
            .await?
            .ok_or(OrigaError::UserNotFound { user_id })?;

        user.set_current_japanese_level(current_japanese_level);
        user.set_native_language(native_language);
        user.set_duolingo_jwt_token(duolingo_jwt_token);
        user.set_telegram_user_id(telegram_user_id);
        user.set_reminders_enabled(reminders_enabled);

        self.repository.save(&user).await?;

        info!(user_id = %user_id, "User profile updated");
        Ok(())
    }
}
