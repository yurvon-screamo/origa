use crate::components::interactive::flash_card::{
    GrammarCard, KanjiCard, StudyCard, StudyCardWrapper, VocabCard, VocabExample,
};
use origa::domain::{Card, OrigaError, Rating};
use ulid::Ulid;

#[derive(Clone)]
pub struct StudyService;

impl StudyService {
    pub fn new() -> Self {
        Self {}
    }

    /// Получить карточки для урока
    pub async fn get_lesson_cards(
        &self,
        _user_id: Ulid,
    ) -> Result<Vec<StudyCardWrapper>, OrigaError> {
        // TODO: Интегрировать SelectCardsToLessonUseCase когда будет repository
        // let use_case = SelectCardsToLessonUseCase::new(repository);
        // let cards = use_case.execute(user_id).await?;
        // self.convert_cards_to_wrappers(cards)

        // Временно возвращаем mock данные
        Ok(self.create_mock_cards())
    }

    /// Получить карточки для закрепления
    pub async fn get_fixation_cards(
        &self,
        _user_id: Ulid,
    ) -> Result<Vec<StudyCardWrapper>, OrigaError> {
        // TODO: Интегрировать SelectCardsToFixationUseCase когда будет repository
        // let use_case = SelectCardsToFixationUseCase::new(repository);
        // let cards = use_case.execute(user_id).await?;
        // self.convert_cards_to_wrappers(cards)

        Ok(self.create_mock_cards())
    }

    /// Оценить карточку
    pub async fn rate_card(
        &self,
        _user_id: Ulid,
        _card_id: Ulid,
        _rating: Rating,
        _is_fixation: bool,
    ) -> Result<(), OrigaError> {
        // TODO: Интегрировать RateCardUseCase когда будет repository и srs_service
        // let mode = if is_fixation { RateMode::Fixation } else { RateMode::Lesson };
        // let use_case = RateCardUseCase::new(repository, srs_service);
        // use_case.execute(user_id, card_id, mode, rating).await

        Ok(())
    }

    /// Завершить урок
    pub async fn complete_lesson(
        &self,
        _user_id: Ulid,
        _duration_seconds: u64,
    ) -> Result<(), OrigaError> {
        // TODO: Интегрировать CompleteLessonUseCase когда будет repository
        // let use_case = CompleteLessonUseCase::new(repository);
        // use_case.execute(user_id, duration_seconds).await

        Ok(())
    }

    fn convert_card_to_wrapper(&self, card: &Card) -> StudyCardWrapper {
        match card {
            Card::Vocabulary(vocab) => StudyCardWrapper {
                card: StudyCard::Vocab(VocabCard {
                    japanese: vocab.word().text().to_string(),
                    reading: "".to_string(), // TODO: Extract reading from word if available
                    translation: vocab.meaning().text().to_string(),
                    examples: vocab
                        .example_phrases()
                        .iter()
                        .map(|ex| VocabExample {
                            japanese: ex.text().to_string(),
                            reading: "".to_string(), // TODO: Extract reading if available
                            translation: ex.translation().to_string(),
                        })
                        .collect(),
                }),
            },
            Card::Kanji(kanji) => StudyCardWrapper {
                card: StudyCard::Kanji(KanjiCard {
                    character: kanji.kanji().text().to_string(),
                    stroke_count: 0,
                    meanings: vec![kanji.description().text().to_string()],
                    onyomi: vec![],
                    kunyomi: vec![],
                    radicals: vec![],
                }),
            },
            Card::Grammar(grammar) => StudyCardWrapper {
                card: StudyCard::Grammar(GrammarCard {
                    pattern: grammar.title().text().to_string(),
                    meaning: grammar.description().text().to_string(),
                    attachment_rules: "".to_string(),
                    examples: vec![],
                }),
            },
        }
    }

    fn create_mock_cards(&self) -> Vec<StudyCardWrapper> {
        vec![
            StudyCardWrapper {
                card: StudyCard::Vocab(VocabCard {
                    japanese: "本".to_string(),
                    reading: "ほん".to_string(),
                    translation: "книга".to_string(),
                    examples: vec![VocabExample {
                        japanese: "本を読みます".to_string(),
                        reading: "ほんをよみます".to_string(),
                        translation: "Я читаю книгу".to_string(),
                    }],
                }),
            },
            StudyCardWrapper {
                card: StudyCard::Kanji(KanjiCard {
                    character: "日".to_string(),
                    stroke_count: 4,
                    meanings: vec!["день".to_string(), "солнце".to_string()],
                    onyomi: vec!["ニチ".to_string()],
                    kunyomi: vec!["ひ".to_string()],
                    radicals: vec![],
                }),
            },
        ]
    }
}
