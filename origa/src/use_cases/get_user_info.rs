use tracing::{debug, info};
use ulid::Ulid;

use crate::domain::DailyHistoryItem;
use crate::domain::JlptProgress;
use crate::domain::OrigaError;
use crate::domain::{JapaneseLevel, NativeLanguage};
use crate::traits::UserRepository;

#[derive(Clone, Debug)]
pub struct UserProfile {
    pub id: Ulid,
    pub username: String,
    pub current_japanese_level: JapaneseLevel,
    pub native_language: NativeLanguage,
    pub jlpt_progress: JlptProgress,
    pub lesson_history: Vec<DailyHistoryItem>,
    pub telegram_user_id: Option<u64>,
}

#[derive(Clone)]
pub struct GetUserInfoUseCase<'a, R: UserRepository> {
    repository: &'a R,
}

impl<'a, R: UserRepository> GetUserInfoUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    pub async fn execute(&self) -> Result<UserProfile, OrigaError> {
        debug!("Getting user info");

        let user = self
            .repository
            .get_current_user()
            .await?
            .ok_or(OrigaError::CurrentUserNotExist {})?;

        info!("User info retrieved successfully");

        Ok(UserProfile {
            id: user.id(),
            username: user.username().to_string(),
            current_japanese_level: user.current_japanese_level(),
            native_language: *user.native_language(),
            jlpt_progress: user.jlpt_progress().clone(),
            lesson_history: user.knowledge_set().lesson_history().to_vec(),
            telegram_user_id: user.telegram_user_id().copied(),
        })
    }
}
