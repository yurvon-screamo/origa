use std::collections::HashMap;

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::application::jlpt_content_loader::JlptContent;
use crate::domain::{
    Card, JapaneseLevel, JlptProgress, KnowledgeSet, MemoryState, NativeLanguage, OrigaError,
    Rating, StudyCard,
    score_content::{ScoreContentResult, score_content},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    id: Ulid,
    email: String,
    username: String,
    native_language: NativeLanguage,
    jlpt_progress: JlptProgress,
    duolingo_jwt_token: Option<String>,
    telegram_user_id: Option<u64>,
    knowledge_set: KnowledgeSet,
    reminders_enabled: bool,

    #[serde(default)]
    updated_at: DateTime<Utc>,
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
            duolingo_jwt_token: None,
            telegram_user_id,
            reminders_enabled: true,
            updated_at: Utc::now(),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn from_row(
        id: Ulid,
        email: String,
        username: String,
        jlpt_progress: JlptProgress,
        native_language: NativeLanguage,
        duolingo_jwt_token: Option<String>,
        telegram_user_id: Option<u64>,
        reminders_enabled: bool,
        knowledge_set: KnowledgeSet,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            email,
            username,
            jlpt_progress,
            native_language,
            duolingo_jwt_token,
            telegram_user_id,
            reminders_enabled,
            knowledge_set,
            updated_at,
        }
    }

    pub fn merge(&mut self, new_values: &User) {
        self.email = new_values.email.clone();
        self.username = new_values.username.clone();
        self.native_language = new_values.native_language.clone();
        self.jlpt_progress = new_values.jlpt_progress.clone();
        self.duolingo_jwt_token = new_values.duolingo_jwt_token.clone();
        self.telegram_user_id = new_values.telegram_user_id;
        self.reminders_enabled = new_values.reminders_enabled;
        self.knowledge_set.merge(&new_values.knowledge_set);
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

    pub fn duolingo_jwt_token(&self) -> Option<&str> {
        self.duolingo_jwt_token.as_deref()
    }

    pub fn set_duolingo_jwt_token(&mut self, token: Option<String>) {
        self.duolingo_jwt_token = token;
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

    pub fn rate_card(
        &mut self,
        card_id: Ulid,
        rating: Rating,
        interval: Duration,
        memory_state: MemoryState,
    ) -> Result<(), OrigaError> {
        self.knowledge_set
            .rate_card(card_id, rating, interval, memory_state)?;
        Ok(())
    }

    pub fn delete_card(&mut self, card_id: Ulid) -> Result<(), OrigaError> {
        self.knowledge_set.delete_card(card_id)
    }

    pub fn create_card(&mut self, card: Card) -> Result<StudyCard, OrigaError> {
        self.knowledge_set.create_card(card)
    }

    pub fn score_content(&self, content: &str) -> Result<ScoreContentResult, OrigaError> {
        score_content(content, self.knowledge_set().study_cards())
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
    use crate::domain::VocabularyCard;
    use crate::domain::value_objects::{Answer, Question};

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

        let memory_state = MemoryState::new(
            crate::domain::memory::Stability::new(15.0).unwrap(),
            crate::domain::memory::Difficulty::new(0.1).unwrap(),
            Utc::now(),
        );

        user.rate_card(
            *study_card1.card_id(),
            Rating::Easy,
            Duration::days(1),
            memory_state,
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
        let memory_state = MemoryState::new(
            crate::domain::memory::Stability::new(15.0).unwrap(),
            crate::domain::memory::Difficulty::new(0.1).unwrap(),
            Utc::now(),
        );
        user.rate_card(
            *study_card.card_id(),
            Rating::Easy,
            Duration::days(1),
            memory_state,
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
}
