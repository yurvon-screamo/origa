use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::domain::{
    Card, CardType, DailyLoad, JapaneseLevel, JlptContent, JlptProgress, KnowledgeSet,
    NativeLanguage, OrigaError, RateMode, Rating, ScoreContentResult, StudyCard, get_translation,
    score_content,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordKnowledge {
    pub is_known: bool,
    pub meaning: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    id: Ulid,
    email: String,
    username: String,
    native_language: NativeLanguage,
    jlpt_progress: JlptProgress,
    telegram_user_id: Option<u64>,
    knowledge_set: KnowledgeSet,

    #[serde(default)]
    updated_at: DateTime<Utc>,

    #[serde(default)]
    imported_sets: HashSet<String>,

    #[serde(default)]
    daily_load: DailyLoad,

    #[serde(default)]
    phrases_seeded: bool,
}

impl User {
    pub fn new(
        email: String,
        native_language: NativeLanguage,
        telegram_user_id: Option<u64>,
    ) -> Self {
        Self {
            id: Ulid::new(),
            username: email.split('@').next().unwrap_or(&email).to_string(),
            email,
            knowledge_set: KnowledgeSet::new(),
            jlpt_progress: JlptProgress::new(),
            native_language,
            telegram_user_id,
            updated_at: Utc::now(),
            imported_sets: HashSet::new(),
            daily_load: DailyLoad::default(),
            phrases_seeded: false,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn from_row(
        id: Ulid,
        email: String,
        username: String,
        jlpt_progress: JlptProgress,
        native_language: NativeLanguage,
        telegram_user_id: Option<u64>,
        knowledge_set: KnowledgeSet,
        updated_at: DateTime<Utc>,
        imported_sets: HashSet<String>,
        daily_load: DailyLoad,
        phrases_seeded: bool,
    ) -> Self {
        Self {
            id,
            email,
            username,
            jlpt_progress,
            native_language,
            telegram_user_id,
            knowledge_set,
            updated_at,
            imported_sets,
            daily_load,
            phrases_seeded,
        }
    }

    pub fn merge(&mut self, another_user: &User) {
        self.email = another_user.email.clone();
        self.username = another_user.username.clone();
        self.native_language = another_user.native_language;
        self.telegram_user_id = another_user.telegram_user_id;
        self.daily_load = another_user.daily_load;

        self.knowledge_set.merge(&another_user.knowledge_set);

        for set_id in &another_user.imported_sets {
            self.imported_sets.insert(set_id.clone());
        }

        self.touch();
    }

    pub fn id(&self) -> Ulid {
        self.id
    }

    pub fn email(&self) -> &str {
        &self.email
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn set_username(&mut self, username: String) {
        self.username = username;
    }

    pub fn current_japanese_level(&self) -> JapaneseLevel {
        self.jlpt_progress.current_level()
    }

    pub fn jlpt_progress(&self) -> &JlptProgress {
        &self.jlpt_progress
    }

    pub fn native_language(&self) -> &NativeLanguage {
        &self.native_language
    }

    pub fn set_native_language(&mut self, native_language: NativeLanguage) {
        self.native_language = native_language
    }

    pub fn knowledge_set(&self) -> &KnowledgeSet {
        &self.knowledge_set
    }

    pub fn telegram_user_id(&self) -> Option<&u64> {
        self.telegram_user_id.as_ref()
    }

    pub fn set_telegram_user_id(&mut self, telegram_user_id: Option<u64>) {
        self.telegram_user_id = telegram_user_id;
    }

    pub fn daily_load(&self) -> &DailyLoad {
        &self.daily_load
    }

    pub fn set_daily_load(&mut self, daily_load: DailyLoad) {
        self.daily_load = daily_load;
        self.touch();
    }

    pub fn updated_at(&self) -> &DateTime<Utc> {
        &self.updated_at
    }

    fn touch(&mut self) {
        self.updated_at = Utc::now();
    }

    pub fn mark_set_as_imported(&mut self, set_id: String) {
        self.imported_sets.insert(set_id);
        self.touch();
    }

    pub fn mark_sets_as_imported(&mut self, set_ids: impl IntoIterator<Item = String>) {
        for set_id in set_ids {
            self.imported_sets.insert(set_id);
        }
        self.touch();
    }

    pub fn is_set_imported(&self, set_id: &str) -> bool {
        self.imported_sets.contains(set_id)
    }

    pub fn imported_sets(&self) -> &HashSet<String> {
        &self.imported_sets
    }

    pub fn phrases_seeded(&self) -> bool {
        self.phrases_seeded
    }

    pub fn set_phrases_seeded(&mut self, value: bool) {
        self.phrases_seeded = value;
        self.touch();
    }

    pub fn is_word_known(&self, word: &str) -> WordKnowledge {
        let meaning = get_translation(word, self.native_language());

        for study_card in self.knowledge_set().study_cards().values() {
            if let Card::Vocabulary(vocab_card) = study_card.card()
                && vocab_card.word().text() == word
            {
                return WordKnowledge {
                    is_known: true,
                    meaning,
                };
            }
        }

        WordKnowledge {
            is_known: false,
            meaning,
        }
    }

    pub fn rate_card(
        &mut self,
        card_id: Ulid,
        rating: Rating,
        mode: RateMode,
    ) -> Result<(), OrigaError> {
        self.knowledge_set.rate_card(card_id, rating, mode)
    }

    pub fn delete_card(&mut self, card_id: Ulid) -> Result<(), OrigaError> {
        self.knowledge_set.delete_card(card_id)
    }

    pub fn create_card(&mut self, card: Card) -> Result<StudyCard, OrigaError> {
        self.knowledge_set.create_card(card)
    }

    pub fn score_content(&self, content: &str) -> Result<ScoreContentResult, OrigaError> {
        score_content::score_content(content, self.knowledge_set().study_cards())
    }

    pub fn toggle_favorite(&mut self, card_id: Ulid) -> Result<(), OrigaError> {
        self.knowledge_set.toggle_favorite(card_id)
    }

    pub fn recalculate_jlpt_progress(&mut self, content: &JlptContent) {
        let mut learned_kanji = HashMap::new();
        let mut learned_words = HashMap::new();
        let mut learned_grammar = HashMap::new();

        for study_card in self.knowledge_set.study_cards().values() {
            if !study_card.memory().is_known_card() {
                continue;
            }

            let card = study_card.card();
            let Some(level) = content.find_level(&card.content_key(), CardType::from(card)) else {
                continue;
            };

            match card {
                Card::Kanji(_) => *learned_kanji.entry(level).or_insert(0) += 1,
                Card::Vocabulary(_) => *learned_words.entry(level).or_insert(0) += 1,
                Card::Grammar(_) => *learned_grammar.entry(level).or_insert(0) += 1,
                Card::Phrase(_) => *learned_words.entry(level).or_insert(0) += 1,
            }
        }

        let total_kanji = Self::build_totals(&content.kanji_by_level);
        let total_words = Self::build_totals(&content.words_by_level);
        let total_grammar = Self::build_totals(&content.grammar_by_level);

        self.jlpt_progress.recalculate(
            &learned_kanji,
            &learned_words,
            &learned_grammar,
            &total_kanji,
            &total_words,
            &total_grammar,
        );
    }

    fn build_totals(
        content: &HashMap<JapaneseLevel, std::collections::HashSet<String>>,
    ) -> HashMap<JapaneseLevel, usize> {
        content
            .iter()
            .map(|(level, set)| (*level, set.len()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_objects::Question;
    use crate::domain::{RateMode, VocabularyCard};

    fn create_test_vocab_card(word: &str) -> Card {
        Card::Vocabulary(VocabularyCard::new(
            Question::new(word.to_string()).unwrap(),
        ))
    }

    fn create_test_content_with_words(words: &[(&str, JapaneseLevel)]) -> JlptContent {
        let mut content = JlptContent::new();
        for (word, level) in words {
            content
                .words_by_level
                .entry(*level)
                .or_default()
                .insert(word.to_string());
        }
        content
    }

    #[test]
    fn user_new_creates_default_jlpt_progress() {
        let user = User::new(
            "test@example.com".to_string(),
            NativeLanguage::Russian,
            None,
        );

        assert_eq!(user.current_japanese_level(), JapaneseLevel::N5);
        assert_eq!(user.email(), "test@example.com");
        assert_eq!(user.username(), "test");
    }

    #[test]
    fn user_current_japanese_level_returns_from_progress() {
        let mut user = User::new(
            "test@example.com".to_string(),
            NativeLanguage::Russian,
            None,
        );

        let complete = crate::domain::jlpt_progress::LevelProgressDetail {
            kanji: crate::domain::jlpt_progress::CategoryProgress {
                learned: 100,
                total: 100,
            },
            words: crate::domain::jlpt_progress::CategoryProgress {
                learned: 100,
                total: 100,
            },
            grammar: crate::domain::jlpt_progress::CategoryProgress {
                learned: 100,
                total: 100,
            },
        };

        user.jlpt_progress.update_level(JapaneseLevel::N5, complete);
        assert_eq!(user.current_japanese_level(), JapaneseLevel::N4);
    }

    #[test]
    fn user_recalculate_jlpt_progress_counts_learned_cards() {
        let mut user = User::new(
            "test@example.com".to_string(),
            NativeLanguage::Russian,
            None,
        );

        let card1 = create_test_vocab_card("猫");
        let card2 = create_test_vocab_card("犬");
        let study_card1 = user.create_card(card1).unwrap();
        let _study_card2 = user.create_card(card2).unwrap();

        user.rate_card(
            *study_card1.card_id(),
            Rating::Easy,
            RateMode::StandardLesson,
        )
        .unwrap();

        let content =
            create_test_content_with_words(&[("猫", JapaneseLevel::N5), ("犬", JapaneseLevel::N5)]);

        user.recalculate_jlpt_progress(&content);

        let n5_progress = user
            .jlpt_progress()
            .level_progress(JapaneseLevel::N5)
            .unwrap();
        assert_eq!(n5_progress.words.learned, 1);
        assert_eq!(n5_progress.words.total, 2);
    }

    #[test]
    fn user_recalculate_jlpt_progress_ignores_unknown_cards() {
        let mut user = User::new(
            "test@example.com".to_string(),
            NativeLanguage::Russian,
            None,
        );

        let card = create_test_vocab_card("猫");
        user.create_card(card).unwrap();

        let content = create_test_content_with_words(&[("猫", JapaneseLevel::N5)]);

        user.recalculate_jlpt_progress(&content);

        let n5_progress = user
            .jlpt_progress()
            .level_progress(JapaneseLevel::N5)
            .unwrap();
        assert_eq!(n5_progress.words.learned, 0);
    }

    #[test]
    fn user_recalculate_jlpt_progress_handles_empty_content() {
        let mut user = User::new(
            "test@example.com".to_string(),
            NativeLanguage::Russian,
            None,
        );

        let card = create_test_vocab_card("猫");
        let study_card = user.create_card(card).unwrap();
        user.rate_card(
            *study_card.card_id(),
            Rating::Easy,
            RateMode::StandardLesson,
        )
        .unwrap();

        let content = JlptContent::new();
        user.recalculate_jlpt_progress(&content);

        let n5_progress = user
            .jlpt_progress()
            .level_progress(JapaneseLevel::N5)
            .unwrap();
        assert_eq!(n5_progress.words.learned, 0);
        assert_eq!(n5_progress.words.total, 0);
    }

    #[test]
    fn user_mark_set_as_imported_adds_to_set() {
        let mut user = User::new(
            "test@example.com".to_string(),
            NativeLanguage::Russian,
            None,
        );

        assert!(!user.is_set_imported("set-123"));

        user.mark_set_as_imported("set-123".to_string());

        assert!(user.is_set_imported("set-123"));
        assert!(user.imported_sets().contains("set-123"));
    }

    #[test]
    fn user_is_set_imported_returns_true_for_imported() {
        let mut user = User::new(
            "test@example.com".to_string(),
            NativeLanguage::Russian,
            None,
        );

        user.mark_set_as_imported("set-abc".to_string());

        assert!(user.is_set_imported("set-abc"));
    }

    #[test]
    fn user_is_set_imported_returns_false_for_not_imported() {
        let user = User::new(
            "test@example.com".to_string(),
            NativeLanguage::Russian,
            None,
        );

        assert!(!user.is_set_imported("set-xyz"));
        assert!(!user.is_set_imported("nonexistent-set"));
    }

    #[test]
    fn user_merge_merges_imported_sets() {
        let mut user1 = User::new(
            "user1@example.com".to_string(),
            NativeLanguage::Russian,
            None,
        );
        user1.mark_set_as_imported("set-1".to_string());

        let mut user2 = User::new(
            "user2@example.com".to_string(),
            NativeLanguage::Russian,
            None,
        );
        user2.mark_set_as_imported("set-2".to_string());
        user2.mark_set_as_imported("set-3".to_string());

        user1.merge(&user2);

        assert!(user1.is_set_imported("set-1"));
        assert!(user1.is_set_imported("set-2"));
        assert!(user1.is_set_imported("set-3"));
        assert_eq!(user1.imported_sets().len(), 3);
    }

    #[test]
    fn user_merge_preserves_jlpt_progress() {
        // Arrange
        let mut user1 = User::new(
            "user1@example.com".to_string(),
            NativeLanguage::Russian,
            None,
        );

        let mut user2 = User::new(
            "user2@example.com".to_string(),
            NativeLanguage::Russian,
            None,
        );

        let complete = crate::domain::jlpt_progress::LevelProgressDetail {
            kanji: crate::domain::jlpt_progress::CategoryProgress {
                learned: 100,
                total: 100,
            },
            words: crate::domain::jlpt_progress::CategoryProgress {
                learned: 100,
                total: 100,
            },
            grammar: crate::domain::jlpt_progress::CategoryProgress {
                learned: 100,
                total: 100,
            },
        };
        user2
            .jlpt_progress
            .update_level(JapaneseLevel::N5, complete);

        let progress_before = user1.jlpt_progress().clone();

        // Act
        user1.merge(&user2);

        // Assert
        assert_eq!(user1.jlpt_progress(), &progress_before);
        assert_eq!(user1.current_japanese_level(), JapaneseLevel::N5);
    }

    #[test]
    fn user_is_word_known_returns_correct_knowledge() {
        let mut user = User::new(
            "test@example.com".to_string(),
            NativeLanguage::Russian,
            None,
        );

        let word = "猫";
        let knowledge_before = user.is_word_known(word);
        assert!(!knowledge_before.is_known);

        let card = create_test_vocab_card(word);
        user.create_card(card).unwrap();

        let knowledge_after = user.is_word_known(word);
        assert!(knowledge_after.is_known);
    }

    #[test]
    fn user_rate_card_nonexistent_returns_card_not_found() {
        // Arrange
        let mut user = User::new(
            "test@example.com".to_string(),
            NativeLanguage::Russian,
            None,
        );
        let nonexistent_id = Ulid::new();

        // Act
        let result = user.rate_card(nonexistent_id, Rating::Good, RateMode::StandardLesson);

        // Assert
        assert!(matches!(result, Err(OrigaError::CardNotFound { .. })));
        if let Err(OrigaError::CardNotFound { card_id }) = result {
            assert_eq!(card_id, nonexistent_id);
        }
    }

    #[test]
    fn user_delete_card_nonexistent_returns_card_not_found() {
        // Arrange
        let mut user = User::new(
            "test@example.com".to_string(),
            NativeLanguage::Russian,
            None,
        );
        let nonexistent_id = Ulid::new();

        // Act
        let result = user.delete_card(nonexistent_id);

        // Assert
        assert!(matches!(result, Err(OrigaError::CardNotFound { .. })));
        if let Err(OrigaError::CardNotFound { card_id }) = result {
            assert_eq!(card_id, nonexistent_id);
        }
    }

    #[test]
    fn user_create_card_duplicate_returns_duplicate_card() {
        // Arrange
        let mut user = User::new(
            "test@example.com".to_string(),
            NativeLanguage::Russian,
            None,
        );
        let word = "猫";
        let card1 = create_test_vocab_card(word);
        let card2 = create_test_vocab_card(word);

        // Act
        let result1 = user.create_card(card1);
        let result2 = user.create_card(card2);

        // Assert
        assert!(result1.is_ok());
        assert!(matches!(result2, Err(OrigaError::DuplicateCard { .. })));
        if let Err(OrigaError::DuplicateCard { question }) = result2 {
            assert_eq!(question, word);
        }
    }

    #[test]
    fn user_toggle_favorite_nonexistent_returns_card_not_found() {
        // Arrange
        let mut user = User::new(
            "test@example.com".to_string(),
            NativeLanguage::Russian,
            None,
        );
        let nonexistent_id = Ulid::new();

        // Act
        let result = user.toggle_favorite(nonexistent_id);

        // Assert
        assert!(matches!(result, Err(OrigaError::CardNotFound { .. })));
        if let Err(OrigaError::CardNotFound { card_id }) = result {
            assert_eq!(card_id, nonexistent_id);
        }
    }

    #[test]
    fn user_rate_card_after_delete_returns_card_not_found() {
        // Arrange
        let mut user = User::new(
            "test@example.com".to_string(),
            NativeLanguage::Russian,
            None,
        );
        let card = create_test_vocab_card("猫");
        let study_card = user.create_card(card).unwrap();
        let card_id = *study_card.card_id();

        // Act
        user.delete_card(card_id).unwrap();
        let result = user.rate_card(card_id, Rating::Good, RateMode::StandardLesson);

        // Assert
        assert!(matches!(result, Err(OrigaError::CardNotFound { .. })));
    }
}
