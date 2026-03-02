use crate::application::user_repository::UserRepository;
use crate::domain::{OrigaError, ScoreContentResult};
use tracing::{debug, info};
use ulid::Ulid;

#[derive(Clone, Copy)]
pub struct ScoreContentUseCase<'a, R: UserRepository> {
    repository: &'a R,
}

impl<'a, R: UserRepository> ScoreContentUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        user_id: Ulid,
        content: &str,
    ) -> Result<ScoreContentResult, OrigaError> {
        debug!(user_id = %user_id, "Scoring content");

        let user = self
            .repository
            .find_by_id(user_id)
            .await?
            .ok_or(OrigaError::UserNotFound { user_id })?;

        let result = user.score_content(content)?;

        info!(
            known_words = result.known_words().len(),
            unknown_words = result.unknown_words().len(),
            known_kanji = result.known_kanji().len(),
            unknown_kanji = result.unknown_kanji().len(),
            "Content scored"
        );

        Ok(result)
    }
}
