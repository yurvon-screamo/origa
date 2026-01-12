use crate::application::user_repository::UserRepository;
use crate::domain::Card;
use crate::domain::OrigaError;
use std::collections::HashMap;
use ulid::Ulid;

#[derive(Clone)]
pub struct SelectCardsToLessonUseCase<'a, R: UserRepository> {
    repository: &'a R,
}

impl<'a, R: UserRepository> SelectCardsToLessonUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, user_id: Ulid) -> Result<HashMap<Ulid, Card>, OrigaError> {
        let user = self
            .repository
            .find_by_id(user_id)
            .await?
            .ok_or(OrigaError::UserNotFound { user_id })?;

        Ok(user.knowledge_set().cards_to_lesson(user.native_language()))
    }
}
