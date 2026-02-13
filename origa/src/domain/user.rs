use chrono::Duration;
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::domain::{
    Card, JapaneseLevel, KnowledgeSet, MemoryState, NativeLanguage, OrigaError, Rating, StudyCard,
    score_content::{ScoreContentResult, score_content},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    id: Ulid,
    username: String,
    native_language: NativeLanguage,
    current_japanese_level: JapaneseLevel,
    duolingo_jwt_token: Option<String>,
    telegram_user_id: Option<u64>,
    knowledge_set: KnowledgeSet,
    reminders_enabled: bool,
}

impl User {
    pub fn new(
        username: String,
        current_japanese_level: JapaneseLevel,
        native_language: NativeLanguage,
        telegram_user_id: Option<u64>,
    ) -> Self {
        Self {
            id: Ulid::new(),
            username,
            knowledge_set: KnowledgeSet::new(),
            current_japanese_level,
            native_language,
            duolingo_jwt_token: None,
            telegram_user_id,
            reminders_enabled: true,
        }
    }

    pub fn id(&self) -> Ulid {
        self.id
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn current_japanese_level(&self) -> &JapaneseLevel {
        &self.current_japanese_level
    }

    pub fn set_current_japanese_level(&mut self, current_japanese_level: JapaneseLevel) {
        self.current_japanese_level = current_japanese_level
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

    pub fn add_lesson_duration(&mut self, lesson_duration: Duration) {
        self.knowledge_set.add_lesson_duration(lesson_duration);
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
}
