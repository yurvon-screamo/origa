use crate::application::UserRepository;
use crate::domain::OrigaError;
use crate::domain::LlmSettings;
use ulid::Ulid;

#[derive(Clone)]
pub struct UpdateUserSettingsUseCase<'a, R: UserRepository> {
    repository: &'a R,
}

#[derive(Clone)]
pub struct UpdateUserSettingsRequest {
    pub llm: Option<LlmSettings>,
    pub duolingo_jwt_token: Option<Option<String>>,
}

impl<'a, R: UserRepository> UpdateUserSettingsUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        user_id: Ulid,
        request: UpdateUserSettingsRequest,
    ) -> Result<(), OrigaError> {
        let mut user = self
            .repository
            .find_by_id(user_id)
            .await?
            .ok_or(OrigaError::UserNotFound { user_id })?;

        let settings = user.settings_mut();

        if let Some(llm) = request.llm {
            settings.set_llm(llm);
        }

        if let Some(duolingo_jwt_token) = request.duolingo_jwt_token {
            settings.set_duolingo_jwt_token(duolingo_jwt_token);
        }

        self.repository.save(&user).await?;
        Ok(())
    }
}
