use crate::domain::OrigaError;
use crate::domain::RateMode;
use crate::domain::Rating;
use crate::traits::UserRepository;
use tracing::{debug, info};
use ulid::Ulid;

#[derive(Clone, Copy)]
pub struct RateCardUseCase<'a, R: UserRepository> {
    repository: &'a R,
}

impl<'a, R: UserRepository> RateCardUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        user_id: Ulid,
        card_id: Ulid,
        mode: RateMode,
        rating: Rating,
    ) -> Result<(), OrigaError> {
        debug!(
            user_id = %user_id,
            card_id = %card_id,
            mode = ?mode,
            rating = ?rating,
            "Rating card"
        );

        let mut user = self
            .repository
            .find_by_id(user_id)
            .await?
            .ok_or(OrigaError::UserNotFound { user_id })?;

        user.rate_card(card_id, rating, mode)?;

        self.repository.save(&user).await?;

        info!(card_id = %card_id, "Card rated");
        Ok(())
    }
}
