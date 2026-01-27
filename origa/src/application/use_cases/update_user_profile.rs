use crate::application::UserRepository;
use crate::domain::{JapaneseLevel, NativeLanguage, OrigaError};
use ulid::Ulid;

#[derive(Clone)]
pub struct UpdateUserProfileUseCase<'a, R: UserRepository> {
    repository: &'a R,
}

#[derive(Clone)]
pub struct UpdateUserProfileRequest {
    pub current_japanese_level: Option<JapaneseLevel>,
    pub native_language: Option<NativeLanguage>,
}

impl<'a, R: UserRepository> UpdateUserProfileUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        user_id: Ulid,
        request: UpdateUserProfileRequest,
    ) -> Result<(), OrigaError> {
        let user = self
            .repository
            .find_by_id(user_id)
            .await?
            .ok_or(OrigaError::UserNotFound { user_id })?;

        // Создать обновленного пользователя через сериализацию/десериализацию
        // Это временное решение до добавления методов set_current_japanese_level и set_native_language в User
        let user_json = serde_json::to_string(&user).map_err(|e| OrigaError::RepositoryError {
            reason: format!("Failed to serialize user: {}", e),
        })?;

        let mut user_map: serde_json::Value =
            serde_json::from_str(&user_json).map_err(|e| OrigaError::RepositoryError {
                reason: format!("Failed to deserialize user: {}", e),
            })?;

        // Обновить поля
        if let Some(level) = request.current_japanese_level {
            user_map["current_japanese_level"] =
                serde_json::to_value(level).map_err(|e| OrigaError::RepositoryError {
                    reason: format!("Failed to serialize level: {}", e),
                })?;
        }

        if let Some(language) = request.native_language {
            user_map["native_language"] =
                serde_json::to_value(language).map_err(|e| OrigaError::RepositoryError {
                    reason: format!("Failed to serialize language: {}", e),
                })?;
        }

        // Десериализовать обратно в User
        let updated_user: crate::domain::User =
            serde_json::from_value(user_map).map_err(|e| OrigaError::RepositoryError {
                reason: format!("Failed to deserialize updated user: {}", e),
            })?;

        self.repository.save(&updated_user).await?;
        Ok(())
    }
}
