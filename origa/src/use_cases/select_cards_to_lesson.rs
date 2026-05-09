use crate::domain::{JlptContent, LessonData, OrigaError};
use crate::traits::UserRepository;
use tracing::{debug, info};

#[derive(Clone)]
pub struct SelectCardsToLessonUseCase<'a, R: UserRepository> {
    repository: &'a R,
}

impl<'a, R: UserRepository> SelectCardsToLessonUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, jlpt_content: &JlptContent) -> Result<LessonData, OrigaError> {
        let user = self
            .repository
            .get_current_user()
            .await?
            .ok_or(OrigaError::CurrentUserNotExist)?;

        debug!(user_id = %user.id(), "Selecting cards to lesson");

        let daily_new_limit = user.daily_load().new_cards_per_day();
        let lesson_data = user
            .knowledge_set()
            .cards_to_lesson(daily_new_limit, jlpt_content);

        info!(user_id = %user.id(), count = lesson_data.total_count(), "Cards selected for lesson");

        Ok(lesson_data)
    }
}
