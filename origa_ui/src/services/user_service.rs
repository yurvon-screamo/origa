use origa::application::{
    GetUserInfoUseCase, KnowledgeSetCardsUseCase, SelectCardsToFixationUseCase,
    SelectCardsToLessonUseCase, UpdateUserProfileRequest, UpdateUserProfileUseCase,
};
use origa::domain::{JapaneseLevel, NativeLanguage, OrigaError};
use origa::settings::ApplicationEnvironment;
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
    pub async fn get_user_profile(&self, user_id: Ulid) -> Result<UserProfileData, OrigaError> {
        let repository = ApplicationEnvironment::get()
            .get_firebase_repository()
            .await?;
        let use_case = GetUserInfoUseCase::new(repository);
        let profile = use_case.execute(user_id).await?;

        Ok(UserProfileData {
            username: profile.username,
            email: format!("{}@origa.local", profile.id), // Email не хранится в User
            current_level: profile.current_japanese_level,
            avatar_url: None, // Avatar не хранится в User
        })
    }

    /// Получить статистику для dashboard
    pub async fn get_dashboard_stats(&self, user_id: Ulid) -> Result<DashboardStats, OrigaError> {
        let repository = ApplicationEnvironment::get()
            .get_firebase_repository()
            .await?;

        // Получить все карточки пользователя
        let knowledge_use_case = KnowledgeSetCardsUseCase::new(repository);
        let all_cards = knowledge_use_case.execute(user_id).await?;

        // Подсчитать статистику по статусам
        let mut learned = 0;
        let mut in_progress = 0;
        let mut new_cards = 0;
        let mut difficult = 0;

        // TODO: get from domain
        for study_card in &all_cards {
            let memory = study_card.memory();
            if memory.is_known_card() {
                learned += 1;
            } else if memory.is_in_progress() {
                in_progress += 1;
            } else if memory.is_new() {
                new_cards += 1;
            } else if memory.is_high_difficulty() {
                difficult += 1;
            }
        }

        // Получить количество карт для урока и закрепления
        let lesson_use_case = SelectCardsToLessonUseCase::new(repository);
        let lesson_cards = lesson_use_case.execute(user_id).await?;

        let fixation_use_case = SelectCardsToFixationUseCase::new(repository);
        let fixation_cards = fixation_use_case.execute(user_id).await?;

        Ok(DashboardStats {
            total_cards: all_cards.len(),
            learned,
            in_progress,
            new_cards,
            difficult,
            lesson_count: lesson_cards.len(),
            fixation_count: fixation_cards.len(),
        })
    }

    /// Обновить уровень JLPT пользователя
    pub async fn update_japanese_level(
        &self,
        user_id: Ulid,
        level: JapaneseLevel,
    ) -> Result<(), OrigaError> {
        let repository = ApplicationEnvironment::get()
            .get_firebase_repository()
            .await?;
        let use_case = UpdateUserProfileUseCase::new(repository);
        let request = UpdateUserProfileRequest {
            current_japanese_level: Some(level),
            native_language: None,
        };
        use_case.execute(user_id, request).await
    }

    /// Обновить язык интерфейса пользователя
    pub async fn update_native_language(
        &self,
        user_id: Ulid,
        language: NativeLanguage,
    ) -> Result<(), OrigaError> {
        let repository = ApplicationEnvironment::get()
            .get_firebase_repository()
            .await?;
        let use_case = UpdateUserProfileUseCase::new(repository);
        let request = UpdateUserProfileRequest {
            current_japanese_level: None,
            native_language: Some(language),
        };
        use_case.execute(user_id, request).await
    }
}

#[derive(Clone, Debug)]
pub struct UserProfileData {
    pub username: String,
    pub email: String,
    pub current_level: JapaneseLevel,
    pub avatar_url: Option<String>,
}
