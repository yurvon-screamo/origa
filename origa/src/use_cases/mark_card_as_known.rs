use crate::domain::{OrigaError, RateMode, Rating};
use crate::traits::UserRepository;
use tracing::{debug, info};
use ulid::Ulid;

#[derive(Clone)]
pub struct MarkCardAsKnownUseCase<'a, R: UserRepository> {
    repository: &'a R,
}

impl<'a, R: UserRepository> MarkCardAsKnownUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, card_id: Ulid) -> Result<(), OrigaError> {
        debug!("Marking card {} as known", card_id);

        let mut user = self
            .repository
            .get_current_user()
            .await?
            .ok_or(OrigaError::CurrentUserNotExist {})?;

        if let Some(study_card) = user.knowledge_set().get_card(card_id) {
            if !study_card.memory().is_new() {
                debug!("Card {} is not new, skipping mark as known", card_id);
                return Ok(());
            }
        }

        user.rate_card(card_id, Rating::Easy, RateMode::StandardLesson)?;

        self.repository.save(&user).await?;

        info!("Card {} marked as known", card_id);
        Ok(())
    }
}
