use crate::domain::OrigaError;
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
        // Повторное «Знаю» на известной карте легитимно: домен обновляет
        // только отметку [marked_known_at] (память нетронута) — гашение
        // компаньонского канала продлевается на текущий день.
        let refresh_only = self.refresh_only(card_id).await?;

        debug!(card_id = %card_id, refresh_only, "Applying known mark");

        let mut user = self
            .repository
            .get_current_user()
            .await?
            .ok_or(OrigaError::CurrentUserNotExist)?;

        user.mark_card_as_known(card_id)?;

        self.repository.save(&user).await?;

        info!(card_id = %card_id, refresh_only, "Known mark applied");
        Ok(())
    }

    async fn refresh_only(&self, card_id: Ulid) -> Result<bool, OrigaError> {
        Ok(self
            .repository
            .get_current_user()
            .await?
            .is_some_and(|user| {
                user.knowledge_set()
                    .get_card(card_id)
                    .is_some_and(|study_card| study_card.memory().is_known_card())
            }))
    }
}
