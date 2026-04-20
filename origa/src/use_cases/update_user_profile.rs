use crate::domain::{DailyLoad, NativeLanguage, OrigaError};
use crate::traits::UserRepository;
use tracing::{debug, info};

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
        native_language: NativeLanguage,
        daily_load: DailyLoad,
        telegram_user_id: Option<u64>,
    ) -> Result<(), OrigaError> {
        debug!("Updating user profile");

        let mut user = self
            .repository
            .get_current_user()
            .await?
            .ok_or(OrigaError::CurrentUserNotExist {})?;

        user.set_native_language(native_language);
        user.set_daily_load(daily_load);
        user.set_telegram_user_id(telegram_user_id);

        self.repository.save_sync(&user).await?;

        info!("User profile updated");
        Ok(())
    }
}
