use origa::application::{
    GetUserInfoUseCase, KnowledgeSetCardsUseCase, SelectCardsToFixationUseCase,
    SelectCardsToLessonUseCase, UpdateUserProfileUseCase,
};
use origa::domain::{JapaneseLevel, NativeLanguage, OrigaError};
use origa::settings::ApplicationEnvironment;

use crate::services::app_services::current_user_id;

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
    pub async fn get_user_profile(&self) -> Result<UserProfileData, OrigaError> {
        let user_id = current_user_id();
        let repository = ApplicationEnvironment::get()
            .get_firebase_repository()
            .await?;
        let use_case = GetUserInfoUseCase::new(repository);
        let profile = use_case.execute(user_id).await?;

        Ok(UserProfileData {
            username: profile.username,
            current_level: profile.current_japanese_level,
            native_language: profile.native_language,
            duolingo_jwt_token: profile.duolingo_jwt_token,
            telegram_user_id: profile.telegram_user_id,
        })
    }

    /// Получить статистику для dashboard
    pub async fn get_dashboard_stats(&self) -> Result<DashboardStats, OrigaError> {
        let user_id = current_user_id();
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
    pub async fn update_japanese_level(&self, level: JapaneseLevel) -> Result<(), OrigaError> {
        let user = self.get_user_profile().await?;
        self.update_profile(
            level,
            user.native_language,
            user.duolingo_jwt_token,
            user.telegram_user_id,
        )
        .await
    }

    /// Обновить язык интерфейса пользователя
    pub async fn update_native_language(&self, language: NativeLanguage) -> Result<(), OrigaError> {
        let user = self.get_user_profile().await?;
        self.update_profile(
            user.current_level,
            language,
            user.duolingo_jwt_token,
            user.telegram_user_id,
        )
        .await
    }

    async fn update_profile(
        &self,
        current_japanese_level: JapaneseLevel,
        native_language: NativeLanguage,
        duolingo_jwt_token: Option<String>,
        telegram_user_id: Option<u64>,
    ) -> Result<(), OrigaError> {
        let user_id = current_user_id();
        let repository = ApplicationEnvironment::get()
            .get_firebase_repository()
            .await?;
        let use_case = UpdateUserProfileUseCase::new(repository);

        use_case
            .execute(
                user_id,
                current_japanese_level,
                native_language,
                duolingo_jwt_token,
                telegram_user_id,
                false, // TODO: add reminders_enabled to UI
            )
            .await
    }
}

#[derive(Clone, Debug)]
pub struct UserProfileData {
    pub username: String,
    pub current_level: JapaneseLevel,
    pub native_language: NativeLanguage,
    pub duolingo_jwt_token: Option<String>,
    pub telegram_user_id: Option<u64>,
}
