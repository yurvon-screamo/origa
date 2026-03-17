use crate::domain::OrigaError;
use crate::traits::UserRepository;
use tracing::{debug, info};
use ulid::Ulid;

#[derive(Clone)]
pub struct DeleteCardUseCase<'a, R: UserRepository> {
    repository: &'a R,
}

impl<'a, R: UserRepository> DeleteCardUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, card_id: Ulid) -> Result<(), OrigaError> {
        debug!(card_id = %card_id, "Deleting card");

        let mut user = self
            .repository
            .get_current_user()
            .await?
            .ok_or(OrigaError::CurrentUserNotExist {})?;

        user.delete_card(card_id)?;

        self.repository.save_sync(&user).await?;

        info!(card_id = %card_id, "Card deleted");
        Ok(())
    }
}
