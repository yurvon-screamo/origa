use crate::domain::LessonCard;
use crate::domain::OrigaError;
use crate::traits::UserRepository;
use std::collections::HashMap;
use tracing::{debug, info};
use ulid::Ulid;

#[derive(Clone)]
pub struct SelectCardsToLessonUseCase<'a, R: UserRepository> {
    repository: &'a R,
}

impl<'a, R: UserRepository> SelectCardsToLessonUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    pub async fn execute(&self) -> Result<HashMap<Ulid, LessonCard>, OrigaError> {
        let user = self
            .repository
            .get_current_user()
            .await?
            .ok_or(OrigaError::CurrentUserNotExist {})?;

        debug!(user_id = %user.id(), "Selecting cards to lesson");

        let daily_new_limit = user.daily_load().new_cards_per_day();
        let cards = user.knowledge_set().cards_to_lesson(daily_new_limit);

        info!(user_id = %user.id(), count = cards.len(), "Cards selected for lesson");

        Ok(cards)
    }
}
