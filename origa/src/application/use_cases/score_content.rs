use crate::application::user_repository::UserRepository;
use crate::domain::{OrigaError, ScoreContentResult};
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
        let user = self
            .repository
            .find_by_id(user_id)
            .await?
            .ok_or(OrigaError::UserNotFound { user_id })?;

        user.score_content(content)
    }
}
