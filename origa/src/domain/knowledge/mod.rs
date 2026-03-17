mod card;
mod daily_history;
mod grammar;
mod kanji;
mod vocabulary;

pub use card::{Card, CardType, GrammarInfo, LessonCardView, QuizCard, QuizOption, StudyCard};
pub use daily_history::DailyHistoryItem;
pub use grammar::GrammarRuleCard;
pub use kanji::{ExampleKanjiWord, KanjiCard};
pub use vocabulary::VocabularyCard;

use std::collections::{HashMap, HashSet};

use crate::domain::{
    get_rule_by_id,
    srs::{rate_memory, NextReview},
    value_objects::NativeLanguage,
    OrigaError, RateMode, Rating, ReviewLog,
};
use chrono::Utc;
use rand::{seq::SliceRandom, Rng};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

const NEW_CARDS_LIMIT: usize = 7;
const HARD_CARDS_LIMIT: usize = 15;

const PROB_NORMAL_VIEW: f32 = 0.25;
const PROB_QUIZ_VIEW: f32 = 0.5;
const PROB_REVERSED_VIEW: f32 = 0.75;

const PROB_KANJI_NORMAL: f32 = 0.33;
const PROB_KANJI_QUIZ: f32 = 0.66;

fn select_applicable_grammar<R: Rng>(
    vocab: &VocabularyCard,
    known_grammars: &[GrammarRuleCard],
    rng: &mut R,
) -> Option<GrammarRuleCard> {
    let word_part = vocab.part_of_speech().ok()?;

    let mut rules: Vec<_> = known_grammars
        .iter()
        .filter(|g| g.apply_to().contains(&word_part))
        .cloned()
        .collect();

    rules.shuffle(rng);
    rules.into_iter().next()
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KnowledgeSet {
    study_cards: HashMap<Ulid, StudyCard>,
    #[serde(default)]
    deleted_cards: HashSet<Ulid>,
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
                }
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
            }
            (Card::Kanji(kanji_card), Card::Kanji(existing_kanji_card)) => {
                kanji_card.kanji() == existing_kanji_card.kanji()
            }
            (Card::Grammar(grammar_rule_card), Card::Grammar(existing_grammar_rule_card)) => {
                grammar_rule_card.rule_id() == existing_grammar_rule_card.rule_id()
            }
            _ => false,
        }) {
            return Err(OrigaError::DuplicateCard {
                question: card.content_key(),
            });
        }

        Ok(())
    }

    pub fn cards_to_fixation(&self, lang: &NativeLanguage) -> HashMap<Ulid, LessonCardView> {
        let mut cards = self
            .study_cards
            .iter()
            .filter(|(_, card)| card.memory().is_high_difficulty())
            .collect::<Vec<_>>();

        cards.sort_by_key(|(_, card)| card.memory().next_review_date());
        cards.reverse();

        cards.truncate(HARD_CARDS_LIMIT);

        let known_rules: Vec<_> = self
            .study_cards
            .values()
            .filter_map(|x| match x.card() {
                Card::Grammar(grammar_rule_card) => Some(grammar_rule_card.clone()),
                _ => None,
            })
            .collect();

        let cards_by_type = self.build_cards_by_type();

        cards
            .iter()
            .map(|(card_id, study_card)| {
                let view = Self::apply_view(study_card, &cards_by_type, &known_rules, lang);
                (**card_id, view)
            })
            .collect()
    }

    pub fn cards_to_lesson(&self, lang: &NativeLanguage) -> HashMap<Ulid, LessonCardView> {
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

        let known_rules: Vec<_> = self
            .study_cards
            .values()
            .filter_map(|x| match x.card() {
                Card::Grammar(grammar_rule_card) => Some(grammar_rule_card.clone()),
                _ => None,
            })
            .collect();

        let cards_by_type = self.build_cards_by_type();

        let mut result: Vec<_> = favorite_cards
            .iter()
            .map(|(card_id, study_card)| {
                let view = Self::apply_view(study_card, &cards_by_type, &known_rules, lang);
                (**card_id, view)
            })
            .collect();

        let priority_shuffled: Vec<_> = priority_cards
            .iter()
            .map(|(card_id, study_card)| {
                let view = Self::apply_view(study_card, &cards_by_type, &known_rules, lang);
                (**card_id, view)
            })
            .collect();

        result.extend(priority_shuffled);
        result.into_iter().collect()
    }

    fn apply_view(
        study_card: &StudyCard,
        cards_by_type: &HashMap<CardType, Vec<Card>>,
        known_grammars: &[GrammarRuleCard],
        lang: &NativeLanguage,
    ) -> LessonCardView {
        Self::apply_view_with_rng(
            study_card,
            cards_by_type,
            known_grammars,
            lang,
            &mut rand::rng(),
        )
    }

    fn apply_view_with_rng<R: Rng>(
        study_card: &StudyCard,
        cards_by_type: &HashMap<CardType, Vec<Card>>,
        known_grammars: &[GrammarRuleCard],
        lang: &NativeLanguage,
        rng: &mut R,
    ) -> LessonCardView {
        let card = study_card.card();
        let card_type = CardType::from(card);
        let is_new = study_card.is_new();

        let same_type_cards = cards_by_type
            .get(&card_type)
            .map(|v| v.as_slice())
            .unwrap_or(&[]);

        match (card_type, is_new) {
            (CardType::Grammar, true) | (CardType::Grammar, false) => {
                LessonCardView::Normal(card.clone())
            }

            (CardType::Kanji, true) | (CardType::Kanji, false) => {
                let rand_val = rng.random::<f32>();
                if rand_val < PROB_KANJI_NORMAL {
                    LessonCardView::Normal(card.clone())
                } else if rand_val < PROB_KANJI_QUIZ {
                    LessonCardView::generate_quiz(card.clone(), same_type_cards, lang)
                        .unwrap_or_else(|_| LessonCardView::Normal(card.clone()))
                } else {
                    LessonCardView::Writing(card.clone())
                }
            }

            (_, true) => {
                if rng.random_bool(0.5) {
                    LessonCardView::generate_quiz(card.clone(), same_type_cards, lang)
                        .unwrap_or_else(|_| LessonCardView::Normal(card.clone()))
                } else {
                    LessonCardView::Normal(card.clone())
                }
            }

            (CardType::Vocabulary, false) => {
                let rand_val = rng.random::<f32>();
                if rand_val < PROB_NORMAL_VIEW {
                    LessonCardView::Normal(card.clone())
                } else if rand_val < PROB_QUIZ_VIEW {
                    LessonCardView::generate_quiz(card.clone(), same_type_cards, lang)
                        .unwrap_or_else(|_| LessonCardView::Normal(card.clone()))
                } else if rand_val < PROB_REVERSED_VIEW {
                    Self::apply_reversed(card, lang)
                } else {
                    Self::apply_grammar_mutated(card, known_grammars, lang, rng)
                }
            }
        }
    }

    fn apply_reversed(card: &Card, lang: &NativeLanguage) -> LessonCardView {
        match card {
            Card::Vocabulary(vocab) => match vocab.revert(lang) {
                Ok(reverted) => LessonCardView::Reversed(Card::Vocabulary(reverted)),
                Err(_) => LessonCardView::Normal(card.clone()),
            },
            _ => LessonCardView::Normal(card.clone()),
        }
    }

    fn apply_grammar_mutated<R: Rng>(
        card: &Card,
        known_grammars: &[GrammarRuleCard],
        lang: &NativeLanguage,
        rng: &mut R,
    ) -> LessonCardView {
        match card {
            Card::Vocabulary(vocab) => {
                match select_applicable_grammar(vocab, known_grammars, rng) {
                    Some(grammar_card) => {
                        let rule = get_rule_by_id(grammar_card.rule_id());
                        match rule {
                            Some(r) => match vocab.with_grammar_rule(r, lang) {
                                Ok((mutated, grammar_description)) => {
                                    let grammar_title = grammar_card
                                        .title(lang)
                                        .map(|q| q.text().to_string())
                                        .unwrap_or_else(|_| grammar_card.rule_id().to_string());
                                    let grammar_info =
                                        GrammarInfo::new(grammar_title, grammar_description);
                                    LessonCardView::GrammarMutated {
                                        card: Card::Vocabulary(mutated),
                                        grammar_info,
                                    }
                                }
                                Err(_) => LessonCardView::Normal(card.clone()),
                            },
                            None => LessonCardView::Normal(card.clone()),
                        }
                    }
                    None => LessonCardView::Normal(card.clone()),
                }
            }
            _ => LessonCardView::Normal(card.clone()),
        }
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
            self.update_history();
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

    fn update_history(&mut self) {
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

        let now = Utc::now();
        let today = now.date_naive();

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
            );
            self.lesson_history.push(item);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::memory::MemoryState;
    use crate::domain::tokenizer::PartOfSpeech;
    use crate::domain::value_objects::Question;
    use crate::use_cases::init_real_dictionaries;
    use chrono::Duration;
    use rand::{rngs::StdRng, SeedableRng};

    fn create_vocab_card(word: &str) -> Card {
        Card::Vocabulary(VocabularyCard::new(
            Question::new(word.to_string()).unwrap(),
        ))
    }

    fn create_kanji_card(kanji: &str, _description: &str) -> Card {
        Card::Kanji(KanjiCard::new_test(kanji.to_string()))
    }

    fn create_grammar_card(_title: &str, _apply_to: Vec<PartOfSpeech>) -> GrammarRuleCard {
        GrammarRuleCard::new_test()
    }

    fn create_study_card_new(card: Card) -> StudyCard {
        StudyCard::new(card)
    }

    fn create_memory_state() -> MemoryState {
        MemoryState::new(
            crate::domain::memory::Stability::new(5.0).unwrap(),
            crate::domain::memory::Difficulty::new(0.5).unwrap(),
            chrono::Utc::now(),
        )
    }

    fn create_known_memory_state() -> MemoryState {
        MemoryState::new(
            crate::domain::memory::Stability::new(15.0).unwrap(),
            crate::domain::memory::Difficulty::new(2.0).unwrap(),
            chrono::Utc::now(),
        )
    }

    #[test]
    fn generate_quiz_returns_normal_for_grammar() {
        let grammar = create_grammar_card("Test Rule", vec![]);
        let lang = NativeLanguage::Russian;
        let result = LessonCardView::generate_quiz(Card::Grammar(grammar), &[], &lang);

        assert!(matches!(result, Ok(LessonCardView::Normal(_))));
    }

    #[test]
    fn generate_quiz_returns_normal_when_not_enough_distractors() {
        init_real_dictionaries();
        let vocab = create_vocab_card("猫");
        let lang = NativeLanguage::Russian;

        let result = LessonCardView::generate_quiz(vocab.clone(), &[], &lang);

        assert!(matches!(result, Ok(LessonCardView::Normal(_))));
    }

    #[test]
    fn lesson_card_view_card_returns_inner_card() {
        let vocab = create_vocab_card("猫");

        let normal = LessonCardView::Normal(vocab.clone());
        assert_eq!(normal.card(), &vocab);

        let reversed = LessonCardView::Reversed(vocab.clone());
        assert_eq!(reversed.card(), &vocab);

        let mutated = LessonCardView::GrammarMutated {
            card: vocab.clone(),
            grammar_info: GrammarInfo::new("Test".to_string(), "Test description".to_string()),
        };
        assert_eq!(mutated.card(), &vocab);

        let quiz = LessonCardView::Quiz(QuizCard::new(vocab.clone(), vec![]));
        assert_eq!(quiz.card(), &vocab);
    }

    #[test]
    fn cards_to_lesson_includes_favorite_cards() {
        let mut knowledge_set = KnowledgeSet::new();
        let card = create_vocab_card("猫");
        let study_card = knowledge_set.create_card(card).unwrap();
        let card_id = *study_card.card_id();

        knowledge_set.toggle_favorite(card_id).unwrap();

        let result = knowledge_set.cards_to_lesson(&NativeLanguage::Russian);
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

        let result = knowledge_set.cards_to_fixation(&NativeLanguage::Russian);

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
    fn grammar_info_new_creates_instance() {
        let info = GrammarInfo::new("Title".to_string(), "Description".to_string());
        assert_eq!(info.title(), "Title");
        assert_eq!(info.description(), "Description");
    }

    #[test]
    fn grammar_info_returns_correct_data() {
        let info = GrammarInfo::new(
            "て-form".to_string(),
            "Форма для соединения глаголов".to_string(),
        );
        assert_eq!(info.title(), "て-form");
        assert_eq!(info.description(), "Форма для соединения глаголов");
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

    #[test]
    fn apply_view_with_rng_kanji_never_reversed_or_grammar_mutated() {
        let kanji = create_kanji_card("日", "день");
        let mut study_kanji = create_study_card_new(kanji);
        study_kanji.add_review(
            create_memory_state(),
            ReviewLog::new(Rating::Good, Duration::days(1)),
        );

        let cards_by_type = HashMap::new();
        let known_grammars = vec![];
        let lang = NativeLanguage::Russian;

        for seed in 0..100u64 {
            let mut rng = StdRng::seed_from_u64(seed);
            let result = KnowledgeSet::apply_view_with_rng(
                &study_kanji,
                &cards_by_type,
                &known_grammars,
                &lang,
                &mut rng,
            );
            assert!(
                !matches!(result, LessonCardView::Reversed(_)),
                "Kanji should never be reversed (seed={seed})"
            );
            assert!(
                !matches!(result, LessonCardView::GrammarMutated { .. }),
                "Kanji should never be grammar mutated (seed={seed})"
            );
        }
    }

    #[test]
    fn apply_view_with_rng_new_cards_never_reversed_or_grammar_mutated() {
        let vocab = create_vocab_card("猫");
        let study_card = create_study_card_new(vocab);

        let cards_by_type = HashMap::new();
        let known_grammars = vec![];
        let lang = NativeLanguage::Russian;

        for seed in 0..100u64 {
            let mut rng = StdRng::seed_from_u64(seed);
            let result = KnowledgeSet::apply_view_with_rng(
                &study_card,
                &cards_by_type,
                &known_grammars,
                &lang,
                &mut rng,
            );
            assert!(
                !matches!(result, LessonCardView::Reversed(_)),
                "New cards should never be reversed (seed={seed})"
            );
            assert!(
                !matches!(result, LessonCardView::GrammarMutated { .. }),
                "New cards should never be grammar mutated (seed={seed})"
            );
        }
    }

    #[test]
    fn apply_view_with_rng_deterministic_results() {
        let vocab = create_vocab_card("猫");
        let mut study_card = create_study_card_new(vocab);
        study_card.add_review(
            create_memory_state(),
            ReviewLog::new(Rating::Good, Duration::days(1)),
        );

        let cards_by_type = HashMap::new();
        let known_grammars = vec![];
        let lang = NativeLanguage::Russian;

        let mut results = Vec::new();
        for seed in 0..10u64 {
            let mut rng = StdRng::seed_from_u64(seed);
            let result = KnowledgeSet::apply_view_with_rng(
                &study_card,
                &cards_by_type,
                &known_grammars,
                &lang,
                &mut rng,
            );
            results.push(format!("{:?}", result));
        }

        let mut results2 = Vec::new();
        for seed in 0..10u64 {
            let mut rng = StdRng::seed_from_u64(seed);
            let result = KnowledgeSet::apply_view_with_rng(
                &study_card,
                &cards_by_type,
                &known_grammars,
                &lang,
                &mut rng,
            );
            results2.push(format!("{:?}", result));
        }

        assert_eq!(
            results, results2,
            "Same seeds should produce identical results"
        );
    }
}
