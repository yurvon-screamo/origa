use crate::components::interactive::flash_card::{
    GrammarCard, KanjiCard, StudyCard, StudyCardWrapper, VocabCard,
};
use crate::services::app_services::current_user_id;
use chrono::Duration;
use origa::application::srs_service::RateMode;
use origa::application::{
    CompleteLessonUseCase, RateCardUseCase, SelectCardsToFixationUseCase,
    SelectCardsToLessonUseCase,
};
use origa::domain::{Card, OrigaError, Rating};
use origa::settings::ApplicationEnvironment;
use ulid::Ulid;

#[derive(Clone)]
pub struct StudyService;

impl StudyService {
    pub fn new() -> Self {
        Self {}
    }

    /// Получить карточки для урока
    pub async fn get_lesson_cards(&self) -> Result<Vec<StudyCardWrapper>, OrigaError> {
        let user_id = current_user_id();
        let repository = ApplicationEnvironment::get()
            .get_firebase_repository()
            .await?;
        let use_case = SelectCardsToLessonUseCase::new(repository);
        let cards_map = use_case.execute(user_id).await?;

        // Конвертировать в StudyCardWrapper
        self.convert_cards_to_wrappers(cards_map)
    }

    /// Получить карточки для закрепления
    pub async fn get_fixation_cards(&self) -> Result<Vec<StudyCardWrapper>, OrigaError> {
        let user_id = current_user_id();
        let repository = ApplicationEnvironment::get()
            .get_firebase_repository()
            .await?;
        let use_case = SelectCardsToFixationUseCase::new(repository);
        let cards_map = use_case.execute(user_id).await?;

        // Конвертировать в StudyCardWrapper
        self.convert_cards_to_wrappers(cards_map)
    }

    /// Оценить карточку
    pub async fn rate_card(
        &self,
        card_id: Ulid,
        rating: Rating,
        is_fixation: bool,
    ) -> Result<(), OrigaError> {
        let user_id = current_user_id();
        let repository = ApplicationEnvironment::get()
            .get_firebase_repository()
            .await?;
        let srs_service = ApplicationEnvironment::get().get_srs_service().await?;
        let mode = if is_fixation {
            RateMode::FixationLesson
        } else {
            RateMode::StandardLesson
        };
        let use_case = RateCardUseCase::new(repository, srs_service);
        use_case.execute(user_id, card_id, mode, rating).await
    }

    /// Завершить урок
    pub async fn complete_lesson(&self, duration_seconds: u64) -> Result<(), OrigaError> {
        let user_id = current_user_id();
        let repository = ApplicationEnvironment::get()
            .get_firebase_repository()
            .await?;
        let use_case = CompleteLessonUseCase::new(repository);
        let duration = Duration::seconds(duration_seconds as i64);
        use_case.execute(user_id, duration).await
    }

    /// Конвертировать HashMap<Ulid, Card> в Vec<StudyCardWrapper>
    fn convert_cards_to_wrappers(
        &self,
        cards_map: std::collections::HashMap<Ulid, Card>,
    ) -> Result<Vec<StudyCardWrapper>, OrigaError> {
        let mut wrappers = Vec::new();

        for (card_id, card) in cards_map {
            let wrapper = self.convert_card_to_wrapper(card_id, &card);
            wrappers.push(wrapper);
        }

        Ok(wrappers)
    }

    fn convert_card_to_wrapper(&self, card_id: Ulid, card: &Card) -> StudyCardWrapper {
        match card {
            Card::Vocabulary(vocab) => StudyCardWrapper {
                card_id,
                card: StudyCard::Vocab(VocabCard {
                    japanese: vocab.word().text().to_string(),
                    translation: vocab.meaning().text().to_string(),
                }),
            },
            Card::Kanji(kanji) => StudyCardWrapper {
                card_id,
                card: StudyCard::Kanji(KanjiCard {
                    character: kanji.kanji().text().to_string(),
                    meanings: vec![kanji.description().text().to_string()],
                    radicals: vec![], // TODO: Get from kanji domain
                }),
            },
            Card::Grammar(grammar) => StudyCardWrapper {
                card_id,
                card: StudyCard::Grammar(GrammarCard {
                    pattern: grammar.title().text().to_string(),
                    meaning: grammar.description().text().to_string(),
                    attachment_rules: grammar
                        .apply_to()
                        .iter()
                        .map(|pos| format!("{:?}", pos))
                        .collect::<Vec<_>>()
                        .join(", "),
                }),
            },
        }
    }
}
