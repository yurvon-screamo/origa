use crate::application::user_repository::UserRepository;
use crate::domain::LessonCardView;
use crate::domain::OrigaError;
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

    pub async fn execute(
        &self,
        user_id: Ulid,
    ) -> Result<HashMap<Ulid, LessonCardView>, OrigaError> {
        debug!(user_id = %user_id, "Selecting cards to lesson");

        let user = self
            .repository
            .find_by_id(user_id)
            .await?
            .ok_or(OrigaError::UserNotFound { user_id })?;

        let cards = user.knowledge_set().cards_to_lesson(user.native_language());
        info!(count = cards.len(), "Cards selected for lesson");

        Ok(cards)
    }
}
