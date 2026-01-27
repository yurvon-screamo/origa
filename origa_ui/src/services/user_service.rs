use origa::application::{GetUserInfoUseCase, UserProfile};
use origa::domain::{JapaneseLevel, OrigaError};
use ulid::Ulid;

#[derive(Clone)]
pub struct UserService;

#[derive(Clone, Debug, Default)]
pub struct DashboardStats {
    pub total_cards: usize,
    pub learned: usize,
    pub in_progress: usize,
    pub new_cards: usize,
    pub difficult: usize,
    pub lesson_count: usize,
    pub fixation_count: usize,
}

impl UserService {
    pub fn new() -> Self {
        Self {}
    }

    /// Получить профиль пользователя
    pub async fn get_user_profile(&self, _user_id: Ulid) -> Result<UserProfileData, OrigaError> {
        // TODO: Интегрировать GetUserInfoUseCase когда будет repository
        // let use_case = GetUserInfoUseCase::new(repository);
        // let profile = use_case.execute(user_id).await?;

        // Временно возвращаем mock данные
        Ok(UserProfileData {
            username: "Изучающий".to_string(),
            email: "user@example.com".to_string(),
            current_level: JapaneseLevel::N5,
            avatar_url: None,
        })
    }

    /// Получить статистику для dashboard
    pub async fn get_dashboard_stats(&self, _user_id: Ulid) -> Result<DashboardStats, OrigaError> {
        // TODO: Получать реальные данные из KnowledgeSetCardsUseCase
        // и SelectCardsToLessonUseCase / SelectCardsToFixationUseCase

        Ok(DashboardStats {
            total_cards: 156,
            learned: 89,
            in_progress: 34,
            new_cards: 33,
            difficult: 12,
            lesson_count: 12,
            fixation_count: 8,
        })
    }
}

#[derive(Clone, Debug)]
pub struct UserProfileData {
    pub username: String,
    pub email: String,
    pub current_level: JapaneseLevel,
    pub avatar_url: Option<String>,
}
