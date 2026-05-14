mod card;
mod daily_history;
mod grammar;
mod kanji;
pub mod lesson;
mod lesson_builder;
mod phrase;
mod stats_updater;
#[cfg(test)]
mod tests;
pub mod vocabulary;

pub use card::{Card, CardType, StudyCard};
pub use daily_history::{DailyHistoryItem, estimate_completion_date};
pub use grammar::GrammarRuleCard;
pub use kanji::{ExampleKanjiWord, KanjiCard};
pub use lesson::{
    GrammarInfo, GrammarQuizCard, LessonCard, LessonCardView, LessonData, LessonViewGenerator,
    MultiQuizResult, QuizCard, QuizMode, QuizOption, YesNoCard,
};
pub use phrase::PhraseCard;
pub use vocabulary::VocabularyCard;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use ulid::Ulid;

use crate::domain::{
    JlptContent, OrigaError, RateMode, Rating, ReviewLog,
    srs::{NextReview, rate_memory},
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KnowledgeSet {
    #[serde(deserialize_with = "deserialize_study_cards")]
    study_cards: HashMap<Ulid, StudyCard>,
    #[serde(default)]
    deleted_cards: HashSet<Ulid>,
    lesson_history: Vec<DailyHistoryItem>,
}

fn deserialize_study_cards<'de, D>(deserializer: D) -> Result<HashMap<Ulid, StudyCard>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct StudyCardsVisitor;

    impl<'de> serde::de::Visitor<'de> for StudyCardsVisitor {
        type Value = HashMap<Ulid, StudyCard>;

        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "a map of study cards")
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::MapAccess<'de>,
        {
            let mut result = HashMap::new();
            while let Some(key) = map.next_key::<Ulid>()? {
                match map.next_value::<StudyCard>() {
                    Ok(value) => {
                        result.insert(key, value);
                    },
                    Err(e) => {
                        tracing::warn!("Skipping study card {}: {}", key, e);
                    },
                }
            }
            Ok(result)
        }
    }

    deserializer.deserialize_map(StudyCardsVisitor)
}

impl Default for KnowledgeSet {
    fn default() -> Self {
        Self::new()
    }
}

impl KnowledgeSet {
    pub fn new() -> Self {
        Self {
            study_cards: HashMap::new(),
            deleted_cards: HashSet::new(),
            lesson_history: Vec::new(),
        }
    }

    pub fn merge(&mut self, new_values: &KnowledgeSet) {
        for deleted_id in &new_values.deleted_cards {
            self.study_cards.remove(deleted_id);
            self.deleted_cards.insert(*deleted_id);
        }

        for (id, study_card) in &new_values.study_cards {
            if self.deleted_cards.contains(id) {
                continue;
            }

            if let Some(existing_card) = self.study_cards.get_mut(id) {
                existing_card.merge(study_card);
            } else if self.validate_unique_card(study_card.card()).is_ok() {
                self.study_cards.insert(*id, study_card.clone());
            }
        }

        for item in &new_values.lesson_history {
            let date = item.timestamp().date_naive();
            if let Some(existing_item) = self
                .lesson_history
                .iter_mut()
                .find(|h| h.timestamp().date_naive() == date)
            {
                existing_item.merge_with(item);
            } else {
                self.lesson_history.push(item.clone());
            }
        }

        self.lesson_history.sort_by_key(|h| h.timestamp());

        self.recalculate_daily_stats();
    }

    pub fn get_card(&self, card_id: Ulid) -> Option<&StudyCard> {
        self.study_cards.get(&card_id)
    }

    pub fn study_cards(&self) -> &HashMap<Ulid, StudyCard> {
        &self.study_cards
    }

    pub fn lesson_history(&self) -> &[DailyHistoryItem] {
        &self.lesson_history
    }

    pub fn new_cards_studied_today(&self) -> usize {
        let today = Utc::now().date_naive();
        self.lesson_history
            .iter()
            .rev()
            .find(|item| item.timestamp().date_naive() == today)
            .map(|item| item.new_cards_studied_today() as usize)
            .unwrap_or(0)
    }

    pub fn phrase_cards_studied_today(&self) -> usize {
        let today = Utc::now().date_naive();
        self.lesson_history
            .iter()
            .rev()
            .find(|item| item.timestamp().date_naive() == today)
            .map(|item| item.phrase_cards_studied_today() as usize)
            .unwrap_or(0)
    }

    pub fn get_known_kanji(&self) -> HashSet<char> {
        self.study_cards
            .values()
            .filter_map(|study_card| match study_card.card() {
                Card::Kanji(kanji_card) if study_card.memory().is_known_card() => {
                    kanji_card.kanji().text().chars().next()
                },
                _ => None,
            })
            .collect()
    }

    pub fn delete_card(&mut self, card_id: Ulid) -> Result<(), OrigaError> {
        if self.study_cards.remove(&card_id).is_none() {
            return Err(OrigaError::CardNotFound { card_id });
        }
        self.deleted_cards.insert(card_id);
        self.recalculate_daily_stats();
        Ok(())
    }

    pub fn deleted_cards(&self) -> &HashSet<Ulid> {
        &self.deleted_cards
    }

    pub fn create_card(&mut self, card: Card) -> Result<StudyCard, OrigaError> {
        let study_card = StudyCard::new(card);
        let card_id = *study_card.card_id();

        self.validate_unique_card(study_card.card())?;

        if self
            .study_cards
            .insert(card_id, study_card.clone())
            .is_some()
        {
            return Err(OrigaError::DuplicateCard {
                question: study_card.card().content_key(),
            });
        }

        self.recalculate_daily_stats();
        Ok(study_card)
    }

    fn build_cards_by_type(&self) -> HashMap<CardType, Vec<Card>> {
        let mut map: HashMap<CardType, Vec<Card>> = HashMap::new();
        for study_card in self.study_cards.values() {
            let card_type = CardType::from(study_card.card());
            map.entry(card_type)
                .or_default()
                .push(study_card.card().clone());
        }
        map
    }

    fn validate_unique_card(&self, card: &Card) -> Result<(), OrigaError> {
        if self.study_cards.values().any(|c| match (card, c.card()) {
            (Card::Vocabulary(vocabulary_card), Card::Vocabulary(existing_vocabulary_card)) => {
                vocabulary_card.word() == existing_vocabulary_card.word()
            },
            (Card::Kanji(kanji_card), Card::Kanji(existing_kanji_card)) => {
                kanji_card.kanji() == existing_kanji_card.kanji()
            },
            (Card::Grammar(grammar_rule_card), Card::Grammar(existing_grammar_rule_card)) => {
                grammar_rule_card.rule_id() == existing_grammar_rule_card.rule_id()
            },
            (Card::Phrase(phrase_card), Card::Phrase(existing_phrase_card)) => {
                phrase_card.phrase_id() == existing_phrase_card.phrase_id()
            },

            _ => false,
        }) {
            return Err(OrigaError::DuplicateCard {
                question: card.content_key(),
            });
        }

        Ok(())
    }

    pub fn cards_to_lesson(
        &self,
        daily_new_limit: usize,
        jlpt_content: &JlptContent,
    ) -> LessonData {
        lesson_builder::build_lesson(self, daily_new_limit, jlpt_content)
    }

    pub(crate) fn rate_card(
        &mut self,
        card_id: Ulid,
        rating: Rating,
        mode: RateMode,
    ) -> Result<(), OrigaError> {
        if let Some(card) = self.study_cards.get_mut(&card_id) {
            let was_new = card.memory().is_new();
            let is_phrase = matches!(card.card(), Card::Phrase(_));
            let effective_mode = if is_phrase {
                RateMode::PhraseReview
            } else {
                mode
            };

            let NextReview {
                interval,
                memory_state,
            } = rate_memory(effective_mode, rating, card.memory())?;

            let review = ReviewLog::new(rating, interval);
            card.add_review(memory_state, review);
            card.handle_favorite_rating(rating);
            self.update_history(rating, was_new, is_phrase);
            Ok(())
        } else {
            Err(OrigaError::CardNotFound { card_id })
        }
    }

    pub(crate) fn toggle_favorite(&mut self, card_id: Ulid) -> Result<(), OrigaError> {
        self.study_cards
            .get_mut(&card_id)
            .map(|card| card.toggle_favorite())
            .ok_or(OrigaError::CardNotFound { card_id })
    }

    fn update_history(&mut self, rating: Rating, was_new: bool, is_phrase: bool) {
        stats_updater::update_history(
            &self.study_cards,
            &mut self.lesson_history,
            rating,
            was_new,
            is_phrase,
        );
    }

    fn recalculate_daily_stats(&mut self) {
        stats_updater::recalculate_daily_stats(&self.study_cards, &mut self.lesson_history);
    }
}
