mod card;
mod daily_history;
mod grammar;
mod kanji;
mod kanji_companions;
pub mod lesson;
mod lesson_builder;
mod phrase;
mod stats_tracker;
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
pub use stats_tracker::StatsTracker;
pub use vocabulary::VocabularyCard;

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use ulid::Ulid;

use crate::dictionary::kanji::get_kanji_info;
use crate::domain::{
    JapaneseLevel, JlptContent, NativeLanguage, OrigaError, RateMode, Rating, ReviewLog,
    srs::{NextReview, rate_memory},
};

pub(crate) const MAX_COMPANION_WORDS: usize = 3;

/// Collects the surface form of every vocabulary card whose memory state
/// qualifies as known. `include_in_progress` widens the predicate to also
/// accept in-progress cards (soft filter used by tail-phrase eligibility and
/// phrase seeding); pass `false` for the strict known-only view.
pub(crate) fn collect_known_vocabulary_words<'a, I>(
    cards: I,
    include_in_progress: bool,
) -> HashSet<String>
where
    I: IntoIterator<Item = &'a StudyCard>,
{
    cards
        .into_iter()
        .filter_map(|sc| match sc.card() {
            Card::Vocabulary(vocab) => {
                let known = sc.memory().is_known_card()
                    || (include_in_progress && sc.memory().is_in_progress());
                known.then(|| vocab.word().text().to_string())
            },
            _ => None,
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KnowledgeSet {
    #[serde(deserialize_with = "deserialize_study_cards")]
    study_cards: HashMap<Ulid, StudyCard>,
    #[serde(default)]
    deleted_cards: HashSet<Ulid>,
    #[serde(flatten)]
    stats: StatsTracker,
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
            stats: StatsTracker::new(),
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

        self.stats.merge(&new_values.stats);
        self.recalculate_daily_stats();
    }

    pub fn get_card(&self, card_id: Ulid) -> Option<&StudyCard> {
        self.study_cards.get(&card_id)
    }

    pub fn study_cards(&self) -> &HashMap<Ulid, StudyCard> {
        &self.study_cards
    }

    pub fn lesson_history(&self) -> &[DailyHistoryItem] {
        self.stats.history()
    }

    pub fn new_cards_studied_today(&self) -> usize {
        self.stats.new_cards_studied_today()
    }

    pub fn phrase_cards_studied_today(&self) -> usize {
        self.stats.phrase_cards_studied_today()
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
        user_level: JapaneseLevel,
    ) -> LessonData {
        use std::collections::HashSet;

        let core = lesson_builder::build_lesson_core(self, daily_new_limit, jlpt_content);
        let with_companions = kanji_companions::add_kanji_companions(core, self, user_level);
        let mut phrase_new_budget = lesson_builder::compute_phrase_new_budget(
            daily_new_limit,
            self.phrase_cards_studied_today(),
        );
        let mut used_phrase_ids: HashSet<Ulid> = HashSet::new();
        let with_interleaved = lesson_builder::add_interleaved_phrases(
            with_companions,
            self,
            &mut used_phrase_ids,
            &mut phrase_new_budget,
        );
        lesson_builder::add_tail_phrases(
            with_interleaved,
            self,
            &used_phrase_ids,
            &mut phrase_new_budget,
        )
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
            let effective_mode = match mode {
                RateMode::ShortTerm | RateMode::OnboardingScoring => mode,
                _ => match card.card() {
                    Card::Phrase(_) => RateMode::PhraseReview,
                    Card::Grammar(_) => RateMode::GrammarReview,
                    Card::Kanji(_) => RateMode::KanjiReview,
                    Card::Vocabulary(_) => mode,
                },
            };

            let NextReview {
                interval,
                memory_state,
            } = rate_memory(effective_mode, rating, card.memory())?;

            let review = ReviewLog::new(rating, interval);
            card.add_review(memory_state, review);
            card.handle_favorite_rating(rating);
            self.update_history(rating, was_new, is_phrase, mode);
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

    fn update_history(&mut self, rating: Rating, was_new: bool, is_phrase: bool, mode: RateMode) {
        self.stats
            .update(&self.study_cards, rating, was_new, is_phrase, mode);
    }

    pub fn create_companion_vocab_cards(
        &mut self,
        kanji_char: &str,
        native_language: &NativeLanguage,
    ) -> Vec<StudyCard> {
        let kanji_info = match get_kanji_info(kanji_char) {
            Ok(info) => info,
            Err(_) => {
                tracing::debug!(kanji = %kanji_char, "Kanji not found, skipping companion creation");
                return Vec::new();
            },
        };

        let mut created = Vec::new();
        for word in kanji_info.popular_words().iter().take(MAX_COMPANION_WORDS) {
            match VocabularyCard::from_known_word(word, native_language) {
                Ok(vocab_card) => match self.create_card(Card::Vocabulary(vocab_card)) {
                    Ok(study_card) => {
                        tracing::debug!(kanji = %kanji_char, word = %word, "Companion vocab card created");
                        created.push(study_card);
                    },
                    Err(OrigaError::DuplicateCard { .. }) => {
                        tracing::debug!(kanji = %kanji_char, word = %word, "Companion already exists, skipping");
                    },
                    Err(e) => {
                        tracing::warn!(kanji = %kanji_char, word = %word, error = %e, "Failed to create companion card");
                    },
                },
                Err(_) => {
                    tracing::debug!(kanji = %kanji_char, word = %word, "No translation for companion word, skipping");
                },
            }
        }
        created
    }

    fn recalculate_daily_stats(&mut self) {
        self.stats.recalculate(&self.study_cards);
    }

    pub fn mark_card_as_known(&mut self, card_id: Ulid) -> Result<(), OrigaError> {
        use crate::domain::memory::{
            Difficulty, KNOWN_CARD_STABILITY_THRESHOLD, MemoryState, Rating, ReviewLog, Stability,
        };
        use chrono::{Duration, Utc};

        if let Some(card) = self.study_cards.get_mut(&card_id) {
            let stability = KNOWN_CARD_STABILITY_THRESHOLD + 1.0;
            let memory = MemoryState::new(
                Stability::new(stability).unwrap(),
                Difficulty::new(3.0).unwrap(),
                Utc::now() - Duration::days(1),
            );
            card.add_review(
                memory,
                ReviewLog::new(Rating::Easy, Duration::days(stability as i64)),
            );
            card.handle_favorite_rating(Rating::Easy);
            Ok(())
        } else {
            Err(OrigaError::CardNotFound { card_id })
        }
    }
}
