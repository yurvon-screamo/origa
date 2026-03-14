use crate::domain::{OrigaError, ScoreContentResult};
use crate::traits::UserRepository;
use tracing::{debug, info};

#[derive(Clone, Copy)]
pub struct ScoreContentUseCase<'a, R: UserRepository> {
    repository: &'a R,
}

impl<'a, R: UserRepository> ScoreContentUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, content: &str) -> Result<ScoreContentResult, OrigaError> {
        debug!("Scoring content");

        let user = self
            .repository
            .get_current_user()
            .await?
            .ok_or(OrigaError::CurrentUserNotExist {})?;

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
