use crate::domain::{
    OrigaError, Rating, ReviewLog,
    knowledge::{GrammarRuleCard, KanjiCard, PhraseCard, VocabularyCard},
    memory::{MemoryHistory, MemoryState},
    value_objects::{Answer, NativeLanguage, Question},
};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StudyCard {
    card_id: Ulid,
    card: Card,
    memory_history: MemoryHistory,
    #[serde(default)]
    is_favorite: bool,
    #[serde(default)]
    perfect_streak_since_known: u8,
}

impl StudyCard {
    pub fn new(card: Card) -> Self {
        Self {
            card_id: Ulid::new(),
            card,
            memory_history: MemoryHistory::default(),
            is_favorite: false,
            perfect_streak_since_known: 0,
        }
    }

    pub fn card_id(&self) -> &Ulid {
        &self.card_id
    }

    pub fn card(&self) -> &Card {
        &self.card
    }

    pub fn memory(&self) -> &MemoryHistory {
        &self.memory_history
    }

    pub fn is_favorite(&self) -> bool {
        self.is_favorite
    }

    pub fn is_new(&self) -> bool {
        self.memory_history.is_new()
    }

    pub fn perfect_streak_since_known(&self) -> u8 {
        self.perfect_streak_since_known
    }

    pub(crate) fn add_review(&mut self, memory_state: MemoryState, review: ReviewLog) {
        self.memory_history.add_review(memory_state, review);
    }

    pub(crate) fn toggle_favorite(&mut self) {
        self.is_favorite = !self.is_favorite;
        if !self.is_favorite {
            self.perfect_streak_since_known = 0;
        }
    }

    pub(crate) fn handle_favorite_rating(&mut self, rating: Rating) {
        if !self.is_favorite || !self.memory_history.is_known_card() {
            return;
        }

        match rating {
            Rating::Easy => {
                self.perfect_streak_since_known += 1;
                if self.perfect_streak_since_known >= 5 {
                    self.is_favorite = false;
                    self.perfect_streak_since_known = 0;
                }
            },
            Rating::Good => {},
            Rating::Hard | Rating::Again => {
                self.perfect_streak_since_known = 0;
            },
        }
    }

    pub fn merge(&mut self, other: &StudyCard) {
        self.memory_history.merge(&other.memory_history);
        self.is_favorite = self.is_favorite || other.is_favorite;
        self.perfect_streak_since_known = self
            .perfect_streak_since_known
            .max(other.perfect_streak_since_known);
    }

    pub fn shuffle_card(&self) -> Card {
        self.card.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Card {
    Vocabulary(VocabularyCard),
    Kanji(KanjiCard),
    Grammar(GrammarRuleCard),
    Phrase(PhraseCard),
}

impl Card {
    pub fn question(&self, lang: &NativeLanguage) -> Result<Question, OrigaError> {
        match self {
            Card::Vocabulary(card) => Ok(card.word().clone()),
            Card::Kanji(card) => Ok(card.kanji().clone()),
            Card::Grammar(card) => card.title(lang),
            Card::Phrase(card) => card.question(),
        }
    }

    pub fn answer(&self, lang: &NativeLanguage) -> Result<Answer, OrigaError> {
        match self {
            Card::Vocabulary(card) => card.answer(lang),
            Card::Kanji(card) => card.description(lang),
            Card::Grammar(card) => card.description(lang),
            Card::Phrase(card) => card.answer(lang),
        }
    }

    pub fn content_key(&self) -> String {
        match self {
            Card::Vocabulary(card) => card.word().text().to_string(),
            Card::Kanji(card) => card.kanji().text().to_string(),
            Card::Grammar(card) => card.rule_id().to_string(),
            Card::Phrase(card) => card.phrase_id().to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CardType {
    Vocabulary,
    Kanji,
    Grammar,
    Phrase,
}

impl From<&Card> for CardType {
    fn from(card: &Card) -> Self {
        match card {
            Card::Vocabulary(_) => CardType::Vocabulary,
            Card::Kanji(_) => CardType::Kanji,
            Card::Grammar(_) => CardType::Grammar,
            Card::Phrase(_) => CardType::Phrase,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_objects::Question;
    use chrono::{Duration, Utc};
    use rstest::rstest;

    fn create_vocabulary_card(word: &str) -> VocabularyCard {
        VocabularyCard::new(Question::new(word.to_string()).unwrap())
    }

    fn create_kanji_card(kanji: &str) -> KanjiCard {
        KanjiCard::new_test(kanji.to_string())
    }

    fn create_grammar_card(rule_id: Ulid) -> GrammarRuleCard {
        GrammarRuleCard::new_test_with_id(rule_id)
    }

    fn create_phrase_card(phrase_id: Ulid) -> PhraseCard {
        PhraseCard::new_test_with_id(phrase_id)
    }

    mod study_card {
        use super::*;

        mod new {
            use super::*;

            #[rstest]
            #[case(Card::Vocabulary(create_vocabulary_card("猫")))]
            #[case(Card::Kanji(create_kanji_card("日")))]
            #[case(Card::Grammar(create_grammar_card(Ulid::new())))]
            #[case(Card::Phrase(create_phrase_card(Ulid::new())))]
            fn creates_study_card_with_card_type(#[case] card: Card) {
                let study_card = StudyCard::new(card);

                assert!(!study_card.card_id().is_nil());
                assert!(study_card.is_new());
                assert!(!study_card.is_favorite());
                assert_eq!(study_card.perfect_streak_since_known(), 0);
            }

            #[test]
            fn creates_unique_card_ids() {
                let card = Card::Vocabulary(create_vocabulary_card("猫"));
                let study_card1 = StudyCard::new(card.clone());
                let study_card2 = StudyCard::new(card);

                assert_ne!(study_card1.card_id(), study_card2.card_id());
            }
        }

        mod card_id {
            use super::*;

            #[test]
            fn returns_valid_ulid() {
                let card = Card::Vocabulary(create_vocabulary_card("猫"));
                let study_card = StudyCard::new(card);

                let card_id = study_card.card_id();

                assert!(!card_id.is_nil());
            }

            #[test]
            fn returns_reference_to_ulid() {
                let card = Card::Vocabulary(create_vocabulary_card("猫"));
                let study_card = StudyCard::new(card);

                let card_id1 = study_card.card_id();
                let card_id2 = study_card.card_id();

                assert_eq!(card_id1, card_id2);
            }
        }

        mod card {
            use super::*;

            #[rstest]
            #[case(Card::Vocabulary(create_vocabulary_card("猫")), CardType::Vocabulary)]
            #[case(Card::Kanji(create_kanji_card("日")), CardType::Kanji)]
            #[case(Card::Grammar(create_grammar_card(Ulid::new())), CardType::Grammar)]
            #[case(Card::Phrase(create_phrase_card(Ulid::new())), CardType::Phrase)]
            fn returns_card_reference(#[case] card: Card, #[case] expected_type: CardType) {
                let study_card = StudyCard::new(card);
                let returned_card = study_card.card();

                assert_eq!(CardType::from(returned_card), expected_type);
            }
        }

        mod memory {
            use super::*;

            #[test]
            fn returns_new_memory_history_for_new_card() {
                let card = Card::Vocabulary(create_vocabulary_card("猫"));
                let study_card = StudyCard::new(card);

                let memory = study_card.memory();

                assert!(memory.is_new());
            }
        }

        mod is_favorite {
            use super::*;

            #[test]
            fn returns_false_for_newly_created_card() {
                let card = Card::Vocabulary(create_vocabulary_card("猫"));
                let study_card = StudyCard::new(card);

                assert!(!study_card.is_favorite());
            }
        }

        mod is_new {
            use super::*;

            #[test]
            fn returns_true_for_newly_created_card() {
                let card = Card::Vocabulary(create_vocabulary_card("猫"));
                let study_card = StudyCard::new(card);

                assert!(study_card.is_new());
            }
        }

        mod perfect_streak_since_known {
            use super::*;

            #[test]
            fn returns_zero_for_new_card() {
                let card = Card::Vocabulary(create_vocabulary_card("猫"));
                let study_card = StudyCard::new(card);

                assert_eq!(study_card.perfect_streak_since_known(), 0);
            }
        }

        mod shuffle_card {
            use super::*;

            #[test]
            fn returns_cloned_card() {
                let vocab_card = create_vocabulary_card("猫");
                let card = Card::Vocabulary(vocab_card.clone());
                let study_card = StudyCard::new(card);

                let shuffled = study_card.shuffle_card();

                assert_eq!(shuffled, Card::Vocabulary(vocab_card));
            }

            #[test]
            fn shuffled_card_is_independent_from_study_card() {
                let vocab_card = create_vocabulary_card("猫");
                let card = Card::Vocabulary(vocab_card);
                let study_card = StudyCard::new(card);

                let _shuffled = study_card.shuffle_card();

                let original_card = study_card.card();
                assert!(matches!(original_card, Card::Vocabulary(_)));
            }
        }

        mod merge {
            use super::*;

            #[test]
            fn merges_memory_history() {
                let card = Card::Vocabulary(create_vocabulary_card("猫"));
                let mut study_card1 = StudyCard::new(card.clone());
                let mut study_card2 = StudyCard::new(card);

                let memory_state = crate::domain::memory::MemoryState::new(
                    crate::domain::memory::Stability::new(10.0).unwrap(),
                    crate::domain::memory::Difficulty::new(2.0).unwrap(),
                    Utc::now(),
                );
                study_card2.add_review(
                    memory_state,
                    crate::domain::memory::ReviewLog::new(
                        crate::domain::memory::Rating::Good,
                        Duration::days(1),
                    ),
                );

                study_card1.merge(&study_card2);

                assert!(!study_card1.is_new());
            }

            #[test]
            fn merges_favorite_status_with_or() {
                let card = Card::Vocabulary(create_vocabulary_card("猫"));
                let mut study_card1 = StudyCard::new(card.clone());
                let mut study_card2 = StudyCard::new(card);

                study_card2.toggle_favorite();

                assert!(!study_card1.is_favorite());

                study_card1.merge(&study_card2);

                assert!(study_card1.is_favorite());
            }

            #[test]
            fn keeps_favorite_if_both_are_favorite() {
                let card = Card::Vocabulary(create_vocabulary_card("猫"));
                let mut study_card1 = StudyCard::new(card.clone());
                let mut study_card2 = StudyCard::new(card);

                study_card1.toggle_favorite();
                study_card2.toggle_favorite();

                assert!(study_card1.is_favorite());

                study_card1.merge(&study_card2);

                assert!(study_card1.is_favorite());
            }

            #[test]
            fn takes_max_perfect_streak_since_known() {
                let card = Card::Vocabulary(create_vocabulary_card("猫"));
                let mut study_card1 = StudyCard::new(card.clone());
                let mut study_card2 = StudyCard::new(card);

                let memory_state = crate::domain::memory::MemoryState::new(
                    crate::domain::memory::Stability::new(15.0).unwrap(),
                    crate::domain::memory::Difficulty::new(2.0).unwrap(),
                    Utc::now(),
                );
                study_card1.add_review(
                    memory_state.clone(),
                    crate::domain::memory::ReviewLog::new(
                        crate::domain::memory::Rating::Good,
                        Duration::days(1),
                    ),
                );
                study_card1.toggle_favorite();
                study_card1.handle_favorite_rating(crate::domain::memory::Rating::Easy);
                study_card1.handle_favorite_rating(crate::domain::memory::Rating::Easy);
                assert_eq!(study_card1.perfect_streak_since_known(), 2);

                study_card2.add_review(
                    memory_state,
                    crate::domain::memory::ReviewLog::new(
                        crate::domain::memory::Rating::Good,
                        Duration::days(1),
                    ),
                );
                study_card2.toggle_favorite();
                study_card2.handle_favorite_rating(crate::domain::memory::Rating::Easy);
                study_card2.handle_favorite_rating(crate::domain::memory::Rating::Easy);
                study_card2.handle_favorite_rating(crate::domain::memory::Rating::Easy);
                assert_eq!(study_card2.perfect_streak_since_known(), 3);

                study_card1.merge(&study_card2);

                assert_eq!(study_card1.perfect_streak_since_known(), 3);
            }
        }

        mod serialization {
            use super::*;

            #[test]
            fn serialization_roundtrip_for_vocabulary_card() {
                let card = Card::Vocabulary(create_vocabulary_card("猫"));
                let study_card = StudyCard::new(card);

                let json = serde_json::to_string(&study_card).unwrap();
                let deserialized: StudyCard = serde_json::from_str(&json).unwrap();

                assert_eq!(study_card.card_id(), deserialized.card_id());
                assert_eq!(study_card.is_favorite(), deserialized.is_favorite());
                assert_eq!(
                    study_card.perfect_streak_since_known(),
                    deserialized.perfect_streak_since_known()
                );
            }

            #[test]
            fn serialization_roundtrip_for_kanji_card() {
                let card = Card::Kanji(create_kanji_card("日"));
                let study_card = StudyCard::new(card);

                let json = serde_json::to_string(&study_card).unwrap();
                let deserialized: StudyCard = serde_json::from_str(&json).unwrap();

                assert_eq!(study_card.card_id(), deserialized.card_id());
            }

            #[test]
            fn serialization_roundtrip_with_favorite_true() {
                let card = Card::Vocabulary(create_vocabulary_card("猫"));
                let mut study_card = StudyCard::new(card);
                study_card.toggle_favorite();

                let json = serde_json::to_string(&study_card).unwrap();
                let deserialized: StudyCard = serde_json::from_str(&json).unwrap();

                assert!(deserialized.is_favorite());
            }

            #[test]
            fn serialization_roundtrip_with_perfect_streak() {
                let card = Card::Vocabulary(create_vocabulary_card("猫"));
                let mut study_card = StudyCard::new(card);

                let memory_state = crate::domain::memory::MemoryState::new(
                    crate::domain::memory::Stability::new(15.0).unwrap(),
                    crate::domain::memory::Difficulty::new(2.0).unwrap(),
                    Utc::now(),
                );
                study_card.add_review(
                    memory_state,
                    crate::domain::memory::ReviewLog::new(
                        crate::domain::memory::Rating::Good,
                        Duration::days(1),
                    ),
                );
                study_card.toggle_favorite();
                study_card.handle_favorite_rating(crate::domain::memory::Rating::Easy);

                let json = serde_json::to_string(&study_card).unwrap();
                let deserialized: StudyCard = serde_json::from_str(&json).unwrap();

                assert_eq!(
                    study_card.perfect_streak_since_known(),
                    deserialized.perfect_streak_since_known()
                );
                assert_eq!(1, deserialized.perfect_streak_since_known());
            }
        }
    }

    mod card {
        use super::*;

        mod question {
            use super::*;

            #[rstest]
            #[case(Card::Vocabulary(create_vocabulary_card("猫")), "猫")]
            #[case(Card::Kanji(create_kanji_card("日")), "日")]
            fn returns_question_for_card(#[case] card: Card, #[case] expected: &str) {
                let question = card.question(&NativeLanguage::Russian);

                assert!(question.is_ok());
                assert_eq!(question.unwrap().text(), expected);
            }
        }

        mod answer {
            use super::*;

            #[test]
            fn returns_answer_for_vocabulary_card() {
                crate::use_cases::init_real_dictionaries();
                let vocab_card = create_vocabulary_card("猫");
                let card = Card::Vocabulary(vocab_card);

                let answer = card.answer(&NativeLanguage::Russian);

                assert!(answer.is_ok());
                let binding = answer.unwrap();
                let answer_text = binding.text();
                assert!(
                    answer_text.contains("кошка") || answer_text.contains("кот"),
                    "Expected answer to contain 'кошка' or 'кот', got: {}",
                    answer_text
                );
            }

            #[test]
            fn returns_answer_for_kanji_card() {
                crate::use_cases::init_real_dictionaries();
                let kanji_card = create_kanji_card("日");
                let card = Card::Kanji(kanji_card);

                let answer = card.answer(&NativeLanguage::Russian);

                assert!(answer.is_ok());
            }
        }

        mod content_key {
            use super::*;

            #[test]
            fn returns_word_for_vocabulary_card() {
                let vocab_card = create_vocabulary_card("猫");
                let card = Card::Vocabulary(vocab_card);

                let content_key = card.content_key();

                assert_eq!(content_key, "猫");
            }

            #[test]
            fn returns_kanji_for_kanji_card() {
                let kanji_card = create_kanji_card("日");
                let card = Card::Kanji(kanji_card);

                let content_key = card.content_key();

                assert_eq!(content_key, "日");
            }

            #[test]
            fn returns_rule_id_for_grammar_card() {
                let rule_id = Ulid::new();
                let grammar_card = create_grammar_card(rule_id);
                let card = Card::Grammar(grammar_card);

                let content_key = card.content_key();

                assert_eq!(content_key, rule_id.to_string());
            }
        }
    }

    mod card_type {
        use super::*;

        mod from_card {
            use super::*;

            #[rstest]
            #[case(Card::Vocabulary(create_vocabulary_card("猫")), CardType::Vocabulary)]
            #[case(Card::Kanji(create_kanji_card("日")), CardType::Kanji)]
            #[case(Card::Grammar(create_grammar_card(Ulid::new())), CardType::Grammar)]
            #[case(Card::Phrase(create_phrase_card(Ulid::new())), CardType::Phrase)]
            fn converts_card_to_type(#[case] card: Card, #[case] expected_type: CardType) {
                let card_type = CardType::from(&card);

                assert_eq!(card_type, expected_type);
            }
        }
    }

    mod card_deserialization {
        use super::*;

        #[test]
        fn unknown_variant_returns_error() {
            let json = r#"{"Radical": {"radical": "一"}}"#;
            let result: Result<Card, _> = serde_json::from_str(json);

            assert!(result.is_err());
            let err_message = result.unwrap_err().to_string();
            assert!(
                err_message.contains("unknown variant `Radical`"),
                "expected unknown variant error, got: {}",
                err_message
            );
        }

        #[test]
        fn known_variants_deserialize_correctly() {
            let vocab = Card::Vocabulary(create_vocabulary_card("猫"));
            let json = serde_json::to_string(&vocab).unwrap();
            let de: Card = serde_json::from_str(&json).unwrap();
            assert_eq!(vocab, de);

            let kanji = Card::Kanji(create_kanji_card("日"));
            let json = serde_json::to_string(&kanji).unwrap();
            let de: Card = serde_json::from_str(&json).unwrap();
            assert_eq!(kanji, de);

            let grammar = Card::Grammar(create_grammar_card(Ulid::new()));
            let json = serde_json::to_string(&grammar).unwrap();
            let de: Card = serde_json::from_str(&json).unwrap();
            assert_eq!(grammar, de);

            let phrase = Card::Phrase(create_phrase_card(Ulid::new()));
            let json = serde_json::to_string(&phrase).unwrap();
            let de: Card = serde_json::from_str(&json).unwrap();
            assert_eq!(phrase, de);
        }
    }
}
