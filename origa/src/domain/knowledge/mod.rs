mod card;
mod daily_history;
mod grammar;
mod kanji;
mod vocabulary;

pub use card::{Card, StudyCard};
pub use daily_history::DailyHistoryItem;
pub use grammar::GrammarRuleCard;
pub use kanji::{ExampleKanjiWord, KanjiCard};
pub use vocabulary::{ExamplePhrase, VocabularyCard};

use std::collections::HashMap;

use crate::domain::{
    OrigaError, Rating, ReviewLog, memory::MemoryState, value_objects::NativeLanguage,
};
use chrono::{Duration, Utc};
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use ulid::Ulid;

const NEW_CARDS_LIMIT: usize = 7;
const HARD_CARDS_LIMIT: usize = 15;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KnowledgeSet {
    study_cards: HashMap<Ulid, StudyCard>,
    lesson_history: Vec<DailyHistoryItem>,
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
            lesson_history: Vec::new(),
        }
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

    pub fn delete_card(&mut self, card_id: Ulid) -> Result<(), OrigaError> {
        if self.study_cards.remove(&card_id).is_none() {
            return Err(OrigaError::CardNotFound { card_id });
        }
        Ok(())
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
                question: study_card.card().question().text().to_string(),
            });
        }

        Ok(study_card)
    }

    fn validate_unique_card(&self, card: &Card) -> Result<(), OrigaError> {
        if self
            .study_cards
            .values()
            .any(|c| c.card().question() == card.question())
        {
            return Err(OrigaError::DuplicateCard {
                question: card.question().text().to_string(),
            });
        }

        Ok(())
    }

    pub fn cards_to_fixation(&self) -> HashMap<Ulid, Card> {
        let mut cards = self
            .study_cards
            .iter()
            .filter(|(_, card)| {
                card.memory().is_low_stability() || card.memory().is_high_difficulty()
            })
            .collect::<Vec<_>>();

        cards.sort_by_key(|(_, card)| card.memory().next_review_date());
        cards.reverse();

        cards.truncate(HARD_CARDS_LIMIT);

        cards
            .iter()
            .map(|(card_id, card)| (**card_id, card.card().clone()))
            .collect()
    }

    pub fn cards_to_lesson(&self, lang: &NativeLanguage) -> HashMap<Ulid, Card> {
        let mut all_cards = self.study_cards.iter().collect::<Vec<_>>();
        all_cards.sort_by_key(|(_, card)| card.memory().next_review_date());

        let mut priority_cards: Vec<_> = all_cards
            .iter()
            .filter(|(_, card)| {
                card.memory().is_due()
                    && (card.memory().is_low_stability() || card.memory().is_high_difficulty())
            })
            .collect();

        if priority_cards.len() < NEW_CARDS_LIMIT {
            let allowed_new = NEW_CARDS_LIMIT.saturating_sub(priority_cards.len());
            let new_cards = all_cards
                .iter()
                .filter(|(_, card)| card.memory().is_new())
                .take(allowed_new);

            priority_cards.extend(new_cards);
        }

        let known_cards = all_cards.iter().filter(|(_, card)| {
            card.memory().is_due()
                && (card.memory().is_in_progress() || card.memory().is_known_card())
        });

        priority_cards.extend(known_cards);
        priority_cards.shuffle(&mut rand::rng());

        let known_rules: Vec<_> = self
            .study_cards
            .values()
            .filter_map(|x| match x.card() {
                Card::Grammar(grammar_rule_card) => Some(grammar_rule_card.clone()),
                _ => None,
            })
            .collect();

        priority_cards
            .iter()
            .filter_map(|(card_id, card)| {
                card.shuffle_card(lang, &known_rules)
                    .ok()
                    .map(|c| (**card_id, c))
            })
            .collect()
    }

    pub(crate) fn rate_card(
        &mut self,
        card_id: Ulid,
        rating: Rating,
        interval: Duration,
        memory_state: MemoryState,
    ) -> Result<(), OrigaError> {
        if let Some(card) = self.study_cards.get_mut(&card_id) {
            let review = ReviewLog::new(rating, interval);
            card.add_review(memory_state, review);
            Ok(())
        } else {
            Err(OrigaError::CardNotFound { card_id })
        }
    }

    pub(crate) fn add_lesson_duration(&mut self, lesson_duration: Duration) {
        self.update_history();
        if let Some(last_item) = self.lesson_history.last_mut() {
            last_item.add_lesson_duration(lesson_duration);
        }
    }

    fn update_history(&mut self) {
        let stability_cards: Vec<_> = self
            .study_cards
            .values()
            .filter_map(|card| card.memory().stability())
            .collect();

        let avg_stability = if stability_cards.is_empty() {
            None
        } else {
            Some(
                stability_cards
                    .iter()
                    .map(|stability| stability.value())
                    .sum::<f64>()
                    / stability_cards.len() as f64,
            )
        };

        let difficulty_cards: Vec<_> = self
            .study_cards
            .values()
            .filter_map(|card| card.memory().difficulty())
            .collect();

        let avg_difficulty = if difficulty_cards.is_empty() {
            None
        } else {
            Some(
                difficulty_cards
                    .iter()
                    .map(|difficulty| difficulty.value())
                    .sum::<f64>()
                    / difficulty_cards.len() as f64,
            )
        };

        let total_words = self.study_cards.len();
        let known_words = self
            .study_cards
            .values()
            .filter(|card| card.memory().is_known_card())
            .count();
        let new_words = self
            .study_cards
            .values()
            .filter(|card| card.memory().is_new())
            .count();
        let in_progress_words = self
            .study_cards
            .values()
            .filter(|card| card.memory().is_in_progress())
            .count();
        let low_stability_words = self
            .study_cards
            .values()
            .filter(|card| card.memory().is_low_stability())
            .count();
        let high_difficulty_words = self
            .study_cards
            .values()
            .filter(|card| card.memory().is_high_difficulty())
            .count();

        let now = Utc::now();
        let today = now.date_naive();

        if let Some(existing_item) = self
            .lesson_history
            .iter_mut()
            .find(|item| item.timestamp().date_naive() == today)
        {
            existing_item.update(
                avg_stability,
                avg_difficulty,
                total_words,
                known_words,
                new_words,
                in_progress_words,
                low_stability_words,
                high_difficulty_words,
            );
        } else {
            let mut item = DailyHistoryItem::new();
            item.update(
                avg_stability,
                avg_difficulty,
                total_words,
                known_words,
                new_words,
                in_progress_words,
                low_stability_words,
                high_difficulty_words,
            );
            self.lesson_history.push(item);
        }
    }
}
