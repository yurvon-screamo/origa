mod card;
mod daily_history;
mod grammar;
mod kanji;
pub mod lesson;

mod vocabulary;

pub use card::{Card, CardType, StudyCard};
pub use daily_history::DailyHistoryItem;
pub use grammar::GrammarRuleCard;
pub use kanji::{ExampleKanjiWord, KanjiCard};
pub use lesson::{
    GrammarInfo, LessonCardView, LessonViewGenerator, QuizCard, QuizOption, YesNoCard,
};

pub use vocabulary::VocabularyCard;

use std::collections::{HashMap, HashSet};

use crate::domain::{
    OrigaError, RateMode, Rating, ReviewLog,
    srs::{NextReview, rate_memory},
};
use chrono::Utc;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use ulid::Ulid;

const NEW_CARDS_LIMIT: usize = 7;
const HARD_CARDS_LIMIT: usize = 15;

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

    pub fn get_known_kanji(&self) -> HashSet<String> {
        self.study_cards
            .values()
            .filter_map(|study_card| match study_card.card() {
                Card::Kanji(kanji_card) if study_card.memory().is_known_card() => {
                    Some(kanji_card.kanji().text().to_string())
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

            _ => false,
        }) {
            return Err(OrigaError::DuplicateCard {
                question: card.content_key(),
            });
        }

        Ok(())
    }

    pub fn cards_to_fixation(&self) -> HashMap<Ulid, LessonCardView> {
        let mut cards = self
            .study_cards
            .iter()
            .filter(|(_, card)| card.memory().is_high_difficulty())
            .collect::<Vec<_>>();

        cards.sort_by_key(|(_, card)| card.memory().next_review_date());
        cards.reverse();

        cards.truncate(HARD_CARDS_LIMIT);

        let generator = LessonViewGenerator::new(self);
        cards
            .iter()
            .map(|(card_id, study_card)| {
                let view = generator.apply_view(study_card, study_card.is_new(), &mut rand::rng());
                (**card_id, view)
            })
            .collect()
    }

    pub fn cards_to_lesson(&self) -> HashMap<Ulid, LessonCardView> {
        let mut all_cards = self.study_cards.iter().collect::<Vec<_>>();
        all_cards.sort_by_key(|(_, card)| card.memory().next_review_date());

        let favorite_cards: Vec<_> = all_cards
            .iter()
            .filter(|(_, card)| card.is_favorite())
            .collect();

        let mut priority_cards: Vec<_> = all_cards
            .iter()
            .filter(|(_, card)| card.memory().is_due() && card.memory().is_high_difficulty())
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

        let generator = LessonViewGenerator::new(self);

        let mut result: Vec<_> = favorite_cards
            .iter()
            .map(|(card_id, study_card)| {
                let view = generator.apply_view(study_card, study_card.is_new(), &mut rand::rng());
                (**card_id, view)
            })
            .collect();

        let priority_shuffled: Vec<_> = priority_cards
            .iter()
            .map(|(card_id, study_card)| {
                let view = generator.apply_view(study_card, study_card.is_new(), &mut rand::rng());
                (**card_id, view)
            })
            .collect();

        result.extend(priority_shuffled);
        result.into_iter().collect()
    }

    pub(crate) fn rate_card(
        &mut self,
        card_id: Ulid,
        rating: Rating,
        mode: RateMode,
    ) -> Result<(), OrigaError> {
        if let Some(card) = self.study_cards.get_mut(&card_id) {
            let NextReview {
                interval,
                memory_state,
            } = rate_memory(mode, rating, card.memory())?;

            let review = ReviewLog::new(rating, interval);
            card.add_review(memory_state, review);
            card.handle_favorite_rating(rating);
            self.update_history(rating);
            Ok(())
        } else {
            Err(OrigaError::CardNotFound { card_id })
        }
    }

    pub(crate) fn toggle_favorite(&mut self, card_id: Ulid) -> Result<(), OrigaError> {
        if let Some(card) = self.study_cards.get_mut(&card_id) {
            card.toggle_favorite();
            Ok(())
        } else {
            Err(OrigaError::CardNotFound { card_id })
        }
    }

    fn update_history(&mut self, rating: Rating) {
        let mut avg_stability = 0.0;
        let mut avg_difficulty = 0.0;
        let mut total_words = 0;
        let mut known_words = 0;
        let mut new_words = 0;
        let mut in_progress_words = 0;
        let mut high_difficulty_words = 0;

        for memory in self.study_cards.values().map(|x| x.memory()) {
            avg_stability += memory.stability().map(|x| x.value()).unwrap_or(0.0);
            avg_difficulty += memory.difficulty().map(|x| x.value()).unwrap_or(0.0);
            total_words += 1;
            known_words += memory.is_known_card() as usize;
            new_words += memory.is_new() as usize;
            in_progress_words += memory.is_in_progress() as usize;
            high_difficulty_words += memory.is_high_difficulty() as usize;
        }

        avg_stability /= total_words as f64;
        avg_difficulty /= total_words as f64;

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
                high_difficulty_words,
                rating,
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
                high_difficulty_words,
                rating,
            );
            self.lesson_history.push(item);
        }
    }

    fn recalculate_daily_stats(&mut self) {
        let mut avg_stability = 0.0;
        let mut avg_difficulty = 0.0;
        let mut total_words = 0;
        let mut known_words = 0;
        let mut new_words = 0;
        let mut in_progress_words = 0;
        let mut high_difficulty_words = 0;

        for memory in self.study_cards.values().map(|x| x.memory()) {
            avg_stability += memory.stability().map(|x| x.value()).unwrap_or(0.0);
            avg_difficulty += memory.difficulty().map(|x| x.value()).unwrap_or(0.0);
            total_words += 1;
            known_words += memory.is_known_card() as usize;
            new_words += memory.is_new() as usize;
            in_progress_words += memory.is_in_progress() as usize;
            high_difficulty_words += memory.is_high_difficulty() as usize;
        }

        if total_words == 0 {
            return;
        }

        avg_stability /= total_words as f64;
        avg_difficulty /= total_words as f64;

        let today = Utc::now().date_naive();
        let (positive, negative, total) = self
            .study_cards
            .values()
            .flat_map(|card| card.memory().reviews())
            .filter(|review| review.timestamp().date_naive() == today)
            .fold((0, 0, 0), |(pos, neg, tot), review| match review.rating() {
                Rating::Easy | Rating::Good => (pos + 1, neg, tot + 1),
                Rating::Hard | Rating::Again => (pos, neg + 1, tot + 1),
            });

        if let Some(existing_item) = self
            .lesson_history
            .iter_mut()
            .find(|item| item.timestamp().date_naive() == today)
        {
            existing_item.update_stats(
                avg_stability,
                avg_difficulty,
                total_words,
                known_words,
                new_words,
                in_progress_words,
                high_difficulty_words,
                positive,
                negative,
                total,
            );
        } else {
            let mut item = DailyHistoryItem::new();
            item.update_stats(
                avg_stability,
                avg_difficulty,
                total_words,
                known_words,
                new_words,
                in_progress_words,
                high_difficulty_words,
                positive,
                negative,
                total,
            );
            self.lesson_history.push(item);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::memory::MemoryState;
    use crate::domain::value_objects::Question;
    use chrono::Duration;

    fn create_vocab_card(word: &str) -> Card {
        Card::Vocabulary(VocabularyCard::new(
            Question::new(word.to_string()).unwrap(),
        ))
    }

    fn create_known_memory_state() -> MemoryState {
        MemoryState::new(
            crate::domain::memory::Stability::new(15.0).unwrap(),
            crate::domain::memory::Difficulty::new(2.0).unwrap(),
            chrono::Utc::now(),
        )
    }

    #[test]
    fn cards_to_lesson_includes_favorite_cards() {
        let mut knowledge_set = KnowledgeSet::new();
        let card = create_vocab_card("猫");
        let study_card = knowledge_set.create_card(card).unwrap();
        let card_id = *study_card.card_id();

        knowledge_set.toggle_favorite(card_id).unwrap();

        let result = knowledge_set.cards_to_lesson();
        assert!(result.contains_key(&card_id));
    }

    #[test]
    fn cards_to_fixation_filters_high_difficulty() {
        let mut knowledge_set = KnowledgeSet::new();

        let card1 = create_vocab_card("猫");
        let card2 = create_vocab_card("犬");

        let study1 = knowledge_set.create_card(card1).unwrap();
        let study2 = knowledge_set.create_card(card2).unwrap();

        knowledge_set
            .rate_card(*study1.card_id(), Rating::Again, RateMode::FixationLesson)
            .unwrap();

        knowledge_set
            .rate_card(*study2.card_id(), Rating::Easy, RateMode::StandardLesson)
            .unwrap();

        let result = knowledge_set.cards_to_fixation();

        assert!(result.contains_key(study1.card_id()));
        assert!(!result.contains_key(study2.card_id()));
    }

    #[test]
    fn handle_favorite_rating_easy_increases_streak() {
        let mut knowledge_set = KnowledgeSet::new();
        let card = create_vocab_card("猫");
        let mut study_card = knowledge_set.create_card(card).unwrap();

        let memory = create_known_memory_state();
        study_card.add_review(
            memory.clone(),
            ReviewLog::new(Rating::Good, Duration::days(1)),
        );
        study_card.toggle_favorite();

        assert_eq!(study_card.perfect_streak_since_known(), 0);
        study_card.handle_favorite_rating(Rating::Easy);
        assert_eq!(study_card.perfect_streak_since_known(), 1);
    }

    #[test]
    fn handle_favorite_rating_good_does_not_change_streak() {
        let mut knowledge_set = KnowledgeSet::new();
        let card = create_vocab_card("猫");
        let mut study_card = knowledge_set.create_card(card).unwrap();

        let memory = create_known_memory_state();
        study_card.add_review(
            memory.clone(),
            ReviewLog::new(Rating::Good, Duration::days(1)),
        );
        study_card.toggle_favorite();

        study_card.handle_favorite_rating(Rating::Easy);
        assert_eq!(study_card.perfect_streak_since_known(), 1);

        study_card.handle_favorite_rating(Rating::Good);
        assert_eq!(study_card.perfect_streak_since_known(), 1);
    }

    #[test]
    fn handle_favorite_rating_hard_resets_streak() {
        let mut knowledge_set = KnowledgeSet::new();
        let card = create_vocab_card("猫");
        let mut study_card = knowledge_set.create_card(card).unwrap();

        let memory = create_known_memory_state();
        study_card.add_review(
            memory.clone(),
            ReviewLog::new(Rating::Good, Duration::days(1)),
        );
        study_card.toggle_favorite();

        study_card.handle_favorite_rating(Rating::Easy);
        assert_eq!(study_card.perfect_streak_since_known(), 1);

        study_card.handle_favorite_rating(Rating::Hard);
        assert_eq!(study_card.perfect_streak_since_known(), 0);
    }

    #[test]
    fn handle_favorite_rating_again_resets_streak() {
        let mut knowledge_set = KnowledgeSet::new();
        let card = create_vocab_card("猫");
        let mut study_card = knowledge_set.create_card(card).unwrap();

        let memory = create_known_memory_state();
        study_card.add_review(
            memory.clone(),
            ReviewLog::new(Rating::Good, Duration::days(1)),
        );
        study_card.toggle_favorite();

        study_card.handle_favorite_rating(Rating::Easy);
        assert_eq!(study_card.perfect_streak_since_known(), 1);

        study_card.handle_favorite_rating(Rating::Again);
        assert_eq!(study_card.perfect_streak_since_known(), 0);
    }

    #[test]
    fn handle_favorite_rating_five_easy_removes_favorite() {
        let mut knowledge_set = KnowledgeSet::new();
        let card = create_vocab_card("猫");
        let mut study_card = knowledge_set.create_card(card).unwrap();

        let memory = create_known_memory_state();
        study_card.add_review(
            memory.clone(),
            ReviewLog::new(Rating::Good, Duration::days(1)),
        );
        study_card.toggle_favorite();

        assert!(study_card.is_favorite());

        for _ in 0..4 {
            study_card.handle_favorite_rating(Rating::Easy);
            assert!(study_card.is_favorite());
        }

        study_card.handle_favorite_rating(Rating::Easy);
        assert!(!study_card.is_favorite());
        assert_eq!(study_card.perfect_streak_since_known(), 0);
    }

    #[test]
    fn handle_favorite_rating_good_does_not_interrupt_accumulation() {
        let mut knowledge_set = KnowledgeSet::new();
        let card = create_vocab_card("猫");
        let mut study_card = knowledge_set.create_card(card).unwrap();

        let memory = create_known_memory_state();
        study_card.add_review(
            memory.clone(),
            ReviewLog::new(Rating::Good, Duration::days(1)),
        );
        study_card.toggle_favorite();

        study_card.handle_favorite_rating(Rating::Easy);
        assert_eq!(study_card.perfect_streak_since_known(), 1);

        study_card.handle_favorite_rating(Rating::Good);
        assert_eq!(study_card.perfect_streak_since_known(), 1);

        study_card.handle_favorite_rating(Rating::Easy);
        assert_eq!(study_card.perfect_streak_since_known(), 2);

        study_card.handle_favorite_rating(Rating::Good);
        assert_eq!(study_card.perfect_streak_since_known(), 2);

        study_card.handle_favorite_rating(Rating::Easy);
        assert_eq!(study_card.perfect_streak_since_known(), 3);

        study_card.handle_favorite_rating(Rating::Good);
        assert_eq!(study_card.perfect_streak_since_known(), 3);

        study_card.handle_favorite_rating(Rating::Easy);
        assert_eq!(study_card.perfect_streak_since_known(), 4);

        study_card.handle_favorite_rating(Rating::Good);
        assert_eq!(study_card.perfect_streak_since_known(), 4);

        study_card.handle_favorite_rating(Rating::Easy);
        assert!(!study_card.is_favorite());
    }

    #[test]
    fn handle_favorite_rating_non_favorited_does_nothing() {
        let mut knowledge_set = KnowledgeSet::new();
        let card = create_vocab_card("猫");
        let mut study_card = knowledge_set.create_card(card).unwrap();

        let memory = create_known_memory_state();
        study_card.add_review(
            memory.clone(),
            ReviewLog::new(Rating::Good, Duration::days(1)),
        );

        assert!(!study_card.is_favorite());

        let initial_streak = study_card.perfect_streak_since_known();
        study_card.handle_favorite_rating(Rating::Easy);
        assert_eq!(study_card.perfect_streak_since_known(), initial_streak);
    }

    #[test]
    fn handle_favorite_rating_unknown_card_does_nothing() {
        let mut knowledge_set = KnowledgeSet::new();
        let card = create_vocab_card("猫");
        let mut study_card = knowledge_set.create_card(card).unwrap();

        study_card.toggle_favorite();

        let initial_streak = study_card.perfect_streak_since_known();
        study_card.handle_favorite_rating(Rating::Easy);
        assert_eq!(study_card.perfect_streak_since_known(), initial_streak);
    }

    #[test]
    fn create_card_updates_daily_stats() {
        let mut knowledge_set = KnowledgeSet::new();

        assert!(knowledge_set.lesson_history().is_empty());

        let card1 = create_vocab_card("猫");
        knowledge_set.create_card(card1).unwrap();

        assert_eq!(knowledge_set.lesson_history().len(), 1);
        let history_item = &knowledge_set.lesson_history()[0];
        assert_eq!(history_item.total_words(), 1);
        assert_eq!(history_item.new_words(), 1);
        assert_eq!(history_item.known_words(), 0);
        assert_eq!(history_item.lessons_completed(), 0);

        let card2 = create_vocab_card("犬");
        knowledge_set.create_card(card2).unwrap();

        assert_eq!(knowledge_set.lesson_history().len(), 1);
        let history_item = &knowledge_set.lesson_history()[0];
        assert_eq!(history_item.total_words(), 2);
        assert_eq!(history_item.new_words(), 2);
        assert_eq!(history_item.lessons_completed(), 0);
    }

    #[test]
    fn merge_respects_tombstones() {
        let mut local = KnowledgeSet::new();
        local.create_card(create_vocab_card("猫")).unwrap();
        let study2 = local.create_card(create_vocab_card("犬")).unwrap();
        local.create_card(create_vocab_card("鳥")).unwrap();

        let remote = local.clone();

        let card2_id = *study2.card_id();
        local.delete_card(card2_id).unwrap();

        assert_eq!(local.study_cards().len(), 2);
        assert!(local.deleted_cards().contains(&card2_id));

        local.merge(&remote);

        assert_eq!(
            local.study_cards().len(),
            2,
            "card2 не должна восстановиться"
        );
        assert!(
            local.deleted_cards().contains(&card2_id),
            "tombstone должен сохраниться"
        );
    }

    #[test]
    fn rate_card_increments_lessons_completed() {
        let mut knowledge_set = KnowledgeSet::new();
        let card = create_vocab_card("猫");
        let study_card = knowledge_set.create_card(card).unwrap();

        knowledge_set
            .rate_card(
                *study_card.card_id(),
                Rating::Good,
                RateMode::StandardLesson,
            )
            .unwrap();

        let history_item = &knowledge_set.lesson_history()[0];
        assert_eq!(history_item.lessons_completed(), 1);
    }

    #[test]
    fn merge_study_cards_updates_existing() {
        let mut local = KnowledgeSet::new();
        let study_card = local.create_card(create_vocab_card("猫")).unwrap();
        let card_id = *study_card.card_id();

        assert!(
            local.get_card(card_id).unwrap().is_new(),
            "карточка должна быть новой до merge"
        );

        let mut remote = local.clone();
        remote
            .rate_card(card_id, Rating::Good, RateMode::StandardLesson)
            .unwrap();

        local.merge(&remote);

        let merged_card = local.get_card(card_id).unwrap();
        assert!(
            !merged_card.is_new(),
            "карточка не должна быть новой после merge"
        );
    }

    #[test]
    fn merge_lessons_completed_takes_max() {
        let mut local = KnowledgeSet::new();
        let card1 = local.create_card(create_vocab_card("猫")).unwrap();
        local
            .rate_card(*card1.card_id(), Rating::Good, RateMode::StandardLesson)
            .unwrap();
        local
            .rate_card(*card1.card_id(), Rating::Good, RateMode::StandardLesson)
            .unwrap();

        let history_item = &local.lesson_history()[0];
        assert_eq!(history_item.lessons_completed(), 2);

        let mut remote = KnowledgeSet::new();
        let card2 = remote.create_card(create_vocab_card("犬")).unwrap();
        remote
            .rate_card(*card2.card_id(), Rating::Good, RateMode::StandardLesson)
            .unwrap();
        remote
            .rate_card(*card2.card_id(), Rating::Good, RateMode::StandardLesson)
            .unwrap();
        remote
            .rate_card(*card2.card_id(), Rating::Good, RateMode::StandardLesson)
            .unwrap();

        let remote_history_item = &remote.lesson_history()[0];
        assert_eq!(remote_history_item.lessons_completed(), 3);

        local.merge(&remote);

        let merged_history = &local.lesson_history()[0];
        assert_eq!(
            merged_history.lessons_completed(),
            3,
            "lessons_completed должен использовать max для идемпотентности"
        );
    }

    #[test]
    fn merge_stats_recalculated_from_actual() {
        let mut local = KnowledgeSet::new();
        for i in 0..100 {
            local
                .create_card(create_vocab_card(&format!("word{i}")))
                .unwrap();
        }

        let mut remote = local.clone();

        for i in 0..50 {
            local
                .create_card(create_vocab_card(&format!("known{i}")))
                .unwrap();
        }

        for i in 0..150 {
            remote
                .create_card(create_vocab_card(&format!("remote{i}")))
                .unwrap();
        }

        local.merge(&remote);

        let history_item = &local.lesson_history()[0];
        assert_eq!(history_item.total_words(), 300);
    }
}
