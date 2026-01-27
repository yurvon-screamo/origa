use crate::components::cards::vocab_card::{CardStatus, VocabularyCardData};
use origa::application::{
    CreateVocabularyCardUseCase, DeleteCardUseCase, KnowledgeSetCardsUseCase,
};
use origa::domain::tokenize_text;
use origa::domain::{Card, OrigaError, StudyCard};
use origa::settings::ApplicationEnvironment;
use ulid::Ulid;

#[derive(Clone)]
pub struct VocabularyService;

impl VocabularyService {
    pub fn new() -> Self {
        Self {}
    }

    /// Получить все vocabulary карточки пользователя
    pub async fn get_user_vocabulary(
        &self,
        user_id: Ulid,
    ) -> Result<Vec<VocabularyCardData>, OrigaError> {
        let repository = ApplicationEnvironment::get().get_repository().await?;
        let use_case = KnowledgeSetCardsUseCase::new(repository);
        let cards = use_case.execute(user_id).await?;

        // Фильтруем только vocabulary карточки и конвертируем
        let vocab_cards: Vec<VocabularyCardData> = cards
            .into_iter()
            .filter_map(|study_card| {
                if let Card::Vocabulary(_) = study_card.card() {
                    Some(self.convert_study_card_to_vocab_data(study_card))
                } else {
                    None
                }
            })
            .collect();

        Ok(vocab_cards)
    }

    /// Создать новую vocabulary карточку
    pub async fn create_vocabulary(
        &self,
        user_id: Ulid,
        japanese: String,
        _translation: String,
    ) -> Result<Vec<StudyCard>, OrigaError> {
        let repository = ApplicationEnvironment::get().get_repository().await?;
        let llm_service = ApplicationEnvironment::get()
            .get_llm_service(user_id)
            .await?;

        // Используем блок для управления lifetime

        {
            let use_case = CreateVocabularyCardUseCase::new(repository, &llm_service);
            use_case.execute(user_id, japanese).await
        }
    }

    /// Удалить карточку
    pub async fn delete_vocabulary(&self, user_id: Ulid, card_id: Ulid) -> Result<(), OrigaError> {
        let repository = ApplicationEnvironment::get().get_repository().await?;
        let use_case = DeleteCardUseCase::new(repository);
        use_case.execute(user_id, card_id).await
    }

    /// Конвертировать StudyCard в VocabularyCardData
    fn convert_study_card_to_vocab_data(&self, study_card: StudyCard) -> VocabularyCardData {
        let card_id = study_card.card_id();
        let memory = study_card.memory();

        // Определить статус карточки
        let status = self.map_memory_to_status(memory);

        // Получить difficulty и stability в процентах (0-100)
        let difficulty = memory
            .difficulty()
            .map(|d| (d.value() * 100.0) as u32)
            .unwrap_or(0);
        let stability = memory
            .stability()
            .map(|s| (s.value() * 100.0) as u32)
            .unwrap_or(0);

        // Получить дату следующего повторения
        let next_review = memory
            .next_review_date()
            .map(|dt| dt.naive_utc())
            .unwrap_or_else(|| chrono::Utc::now().naive_utc());

        // Извлечь данные из Card::Vocabulary
        if let Card::Vocabulary(vocab) = study_card.card() {
            let japanese = vocab.word().text().to_string();

            // Получить reading через tokenizer
            let reading = tokenize_text(&japanese)
                .ok()
                .and_then(|tokens| tokens.first().cloned())
                .map(|token| token.phonological_surface_form().to_string())
                .unwrap_or_default();

            let translation = vocab.meaning().text().to_string();

            VocabularyCardData {
                id: card_id.to_string(),
                japanese,
                reading,
                translation,
                status,
                difficulty,
                stability,
                next_review,
            }
        } else {
            // Fallback (не должно произойти, так как мы фильтруем)
            VocabularyCardData {
                id: card_id.to_string(),
                japanese: String::new(),
                reading: String::new(),
                translation: String::new(),
                status,
                difficulty,
                stability,
                next_review,
            }
        }
    }

    /// Маппинг MemoryHistory -> CardStatus
    fn map_memory_to_status(&self, memory: &origa::domain::MemoryHistory) -> CardStatus {
        if memory.is_new() {
            CardStatus::New
        } else if memory.is_high_difficulty() {
            CardStatus::Difficult
        } else if memory.is_known_card() {
            CardStatus::Mastered
        } else if memory.is_in_progress() {
            CardStatus::InProgress
        } else {
            CardStatus::New // Fallback
        }
    }

    /// Получить статистику по vocabulary
    pub fn get_vocabulary_stats(&self, cards: &[VocabularyCardData]) -> VocabularyStats {
        VocabularyStats {
            total: cards.len(),
            new: cards.iter().filter(|c| c.status == CardStatus::New).count(),
            in_progress: cards
                .iter()
                .filter(|c| c.status == CardStatus::InProgress)
                .count(),
            mastered: cards
                .iter()
                .filter(|c| c.status == CardStatus::Mastered)
                .count(),
            difficult: cards
                .iter()
                .filter(|c| c.status == CardStatus::Difficult)
                .count(),
        }
    }
}

#[derive(Clone, Default)]
pub struct VocabularyStats {
    pub total: usize,
    pub new: usize,
    pub in_progress: usize,
    pub mastered: usize,
    pub difficult: usize,
}
