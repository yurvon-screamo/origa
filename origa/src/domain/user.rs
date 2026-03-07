use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::domain::{
    Card, JapaneseLevel, JlptContent, JlptProgress, KnowledgeSet, NativeLanguage, OrigaError,
    RateMode, Rating, ScoreContentResult, StudyCard, score_content,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    id: Ulid,
    email: String,
    username: String,
    native_language: NativeLanguage,
    jlpt_progress: JlptProgress,
    telegram_user_id: Option<u64>,
    knowledge_set: KnowledgeSet,
    reminders_enabled: bool,

    #[serde(default)]
    updated_at: DateTime<Utc>,

    #[serde(default)]
    imported_sets: HashSet<String>,
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
            reminders_enabled: true,
            updated_at: Utc::now(),
            imported_sets: HashSet::new(),
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
        reminders_enabled: bool,
        knowledge_set: KnowledgeSet,
        updated_at: DateTime<Utc>,
        imported_sets: HashSet<String>,
    ) -> Self {
        Self {
            id,
            email,
            username,
            jlpt_progress,
            native_language,
            telegram_user_id,
            reminders_enabled,
            knowledge_set,
            updated_at,
            imported_sets,
        }
    }

    pub fn merge(&mut self, new_values: &User) {
        self.email = new_values.email.clone();
        self.username = new_values.username.clone();
        self.native_language = new_values.native_language.clone();
        self.jlpt_progress = new_values.jlpt_progress.clone();
        self.telegram_user_id = new_values.telegram_user_id;
        self.reminders_enabled = new_values.reminders_enabled;
        self.knowledge_set.merge(&new_values.knowledge_set);
        for set_id in &new_values.imported_sets {
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

    pub fn reminders_enabled(&self) -> bool {
        self.reminders_enabled
    }

    pub fn set_reminders_enabled(&mut self, reminders_enabled: bool) {
        self.reminders_enabled = reminders_enabled;
    }

    pub fn updated_at(&self) -> &DateTime<Utc> {
        &self.updated_at
    }

    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }

    pub fn mark_set_as_imported(&mut self, set_id: String) {
        self.imported_sets.insert(set_id);
        self.touch();
    }

    pub fn is_set_imported(&self, set_id: &str) -> bool {
        self.imported_sets.contains(set_id)
    }

    pub fn imported_sets(&self) -> &HashSet<String> {
        &self.imported_sets
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
            let key = card.content_key();

            for &level in &[
                JapaneseLevel::N5,
                JapaneseLevel::N4,
                JapaneseLevel::N3,
                JapaneseLevel::N2,
                JapaneseLevel::N1,
            ] {
                match card {
                    crate::domain::Card::Kanji(_) => {
                        if content
                            .kanji_by_level
                            .get(&level)
                            .is_some_and(|set| set.contains(&key))
                        {
                            *learned_kanji.entry(level).or_insert(0) += 1;
                        }
                    }
                    crate::domain::Card::Vocabulary(_) => {
                        if content
                            .words_by_level
                            .get(&level)
                            .is_some_and(|set| set.contains(&key))
                        {
                            *learned_words.entry(level).or_insert(0) += 1;
                        }
                    }
                    crate::domain::Card::Grammar(_) => {
                        if content
                            .grammar_by_level
                            .get(&level)
                            .is_some_and(|set| set.contains(&key))
                        {
                            *learned_grammar.entry(level).or_insert(0) += 1;
                        }
                    }
                }
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
    use crate::domain::value_objects::{Answer, Question};
    use crate::domain::{RateMode, VocabularyCard};

    fn create_test_vocab_card(word: &str) -> Card {
        Card::Vocabulary(VocabularyCard::new(
            Question::new(word.to_string()).unwrap(),
            Answer::new("meaning".to_string()).unwrap(),
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
}
