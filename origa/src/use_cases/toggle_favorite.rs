use crate::domain::OrigaError;
use crate::traits::UserRepository;
use tracing::debug;
use ulid::Ulid;

#[derive(Clone)]
pub struct ToggleFavoriteUseCase<'a, R: UserRepository> {
    repository: &'a R,
}

impl<'a, R: UserRepository> ToggleFavoriteUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, user_id: Ulid, card_id: Ulid) -> Result<bool, OrigaError> {
        debug!(user_id = %user_id, card_id = %card_id, "Toggling favorite");

        let mut user = self
            .repository
            .find_by_id(user_id)
            .await?
            .ok_or(OrigaError::UserNotFound { user_id })?;

        user.toggle_favorite(card_id)?;

        let is_favorite = user
            .knowledge_set()
            .get_card(card_id)
            .map(|c| c.is_favorite())
            .unwrap_or(false);

        self.repository.save(&user).await?;

        Ok(is_favorite)
    }
}
