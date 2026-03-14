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

    pub async fn execute(&self, card_id: Ulid) -> Result<bool, OrigaError> {
        debug!(card_id = %card_id, "Toggling favorite");

        let mut user = self
            .repository
            .get_current_user()
            .await?
            .ok_or(OrigaError::CurrentUserNotExist {})?;

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
