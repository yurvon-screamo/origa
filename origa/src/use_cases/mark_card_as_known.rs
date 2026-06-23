use crate::domain::OrigaError;
use crate::traits::UserRepository;
use tracing::{debug, info, warn};
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
            .ok_or(OrigaError::CurrentUserNotExist)?;

        if let Some(study_card) = user.knowledge_set().get_card(card_id) {
            if study_card.memory().is_known_card() {
                warn!(card_id = %card_id, "Card already learned, skip mark as known");
                return Ok(());
            }
        }

        user.mark_card_as_known(card_id)?;

        self.repository.save(&user).await?;

        info!("Card {} marked as known", card_id);
        Ok(())
    }
}
