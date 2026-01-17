use crate::application::user_repository::UserRepository;
use crate::domain::OrigaError;
use chrono::Duration;
use ulid::Ulid;

#[derive(Clone)]
pub struct CompleteLessonUseCase<'a, R: UserRepository> {
    repository: &'a R,
}

impl<'a, R: UserRepository> CompleteLessonUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        user_id: Ulid,
        lesson_duration: Duration,
    ) -> Result<(), OrigaError> {
        let mut user = self
            .repository
            .find_by_id(user_id)
            .await?
            .ok_or(OrigaError::UserNotFound { user_id })?;

        user.add_lesson_duration(lesson_duration);

        self.repository.save(&user).await?;

        println!("Finished completing lesson: {:?}", lesson_duration);
        Ok(())
    }
}
