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
        debug!("Marking card {} as known", card_id);

        let mut user = self
            .repository
            .get_current_user()
            .await?
            .ok_or(OrigaError::CurrentUserNotExist)?;

        // Повторное «Знаю» на известной карте легитимно: домен обновляет
        // только отметку [marked_known_at] (память нетронута) — гашение
        // компаньонского канала продлевается на текущий день.
        user.mark_card_as_known(card_id)?;

        self.repository.save(&user).await?;

        info!("Card {} marked as known", card_id);
        Ok(())
    }
}
