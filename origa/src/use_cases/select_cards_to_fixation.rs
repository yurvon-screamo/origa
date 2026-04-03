use crate::domain::LessonCardView;
use crate::domain::OrigaError;
use crate::traits::UserRepository;
use std::collections::HashMap;
use tracing::{debug, info};
use ulid::Ulid;

#[derive(Clone)]
pub struct SelectCardsToFixationUseCase<'a, R: UserRepository> {
    repository: &'a R,
}

impl<'a, R: UserRepository> SelectCardsToFixationUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    pub async fn execute(&self) -> Result<HashMap<Ulid, LessonCardView>, OrigaError> {
        debug!("Selecting cards to fixation");

        let user = self
            .repository
            .get_current_user()
            .await?
            .ok_or(OrigaError::CurrentUserNotExist {})?;

        let cards = user.knowledge_set().cards_to_fixation();
        info!(count = cards.len(), "Cards selected for fixation");

        Ok(cards)
    }
}
