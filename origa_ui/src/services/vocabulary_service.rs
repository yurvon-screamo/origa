use crate::components::cards::vocab_card::{CardStatus, VocabularyCardData};
use origa::application::{
    CreateVocabularyCardUseCase, DeleteCardUseCase, KnowledgeSetCardsUseCase,
};
use origa::domain::{Card, OrigaError, StudyCard};
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
        _user_id: Ulid,
    ) -> Result<Vec<VocabularyCardData>, OrigaError> {
        // TODO: Интегрировать KnowledgeSetCardsUseCase когда будет repository
        // let use_case = KnowledgeSetCardsUseCase::new(repository);
        // let cards = use_case.execute(user_id).await?;

        // Временно возвращаем пустой список
        Ok(vec![])
    }

    /// Создать новую vocabulary карточку
    pub async fn create_vocabulary(
        &self,
        _user_id: Ulid,
        japanese: String,
        _translation: String,
    ) -> Result<Vec<StudyCard>, OrigaError> {
        // TODO: Интегрировать CreateVocabularyCardUseCase когда будет repository и llm_service
        // let use_case = CreateVocabularyCardUseCase::new(repository, llm_service);
        // use_case.execute(user_id, japanese).await

        println!("Creating vocabulary: {}", japanese);
        Ok(vec![])
    }

    /// Удалить карточку
    pub async fn delete_vocabulary(
        &self,
        _user_id: Ulid,
        _card_id: Ulid,
    ) -> Result<(), OrigaError> {
        // TODO: Интегрировать DeleteCardUseCase когда будет repository
        // let use_case = DeleteCardUseCase::new(repository);
        // use_case.execute(user_id, card_id).await

        Ok(())
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
