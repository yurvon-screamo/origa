mod memory;
pub use memory::{Difficulty, MemoryHistory, MemoryState, Rating, ReviewLog, Stability};

mod value_objects;
pub use value_objects::{Answer, JapaneseLevel, NativeLanguage, Question};

mod error;
pub use error::KeikakuError;

mod furigana;
pub use furigana::furiganize_text;

mod japanese;
pub use japanese::{JapaneseChar, JapaneseText};

mod well_known_set;
pub use well_known_set::{
    WellKnownSet, WellKnownSetContent, load_jlpt_n1, load_jlpt_n2, load_jlpt_n3, load_jlpt_n4,
    load_jlpt_n5,
};

mod settings;
pub use settings::{LlmSettings, UserSettings};

mod tokenizer;
pub use tokenizer::{PartOfSpeech, TokenInfo, tokenize_text};

mod grammar;
pub use grammar::{
    GRAMMAR_RULES, GrammarRule, GrammarRuleContent, GrammarRuleInfo, get_rule_by_id,
};

mod knowledge;
pub use knowledge::{
    Card, DailyHistoryItem, ExampleKanjiWord, ExamplePhrase, GrammarRuleCard, KanjiCard,
    KnowledgeSet, StudyCard, VocabularyCard,
};

mod dictionary;
pub use dictionary::{
    KANJI_DICTIONARY, KanjiInfo, PopularWord, RADICAL_DICTIONARY, RadicalInfo,
    VOCABULARY_DICTIONARY, VocabularyInfo,
};

use chrono::Duration;
use serde::{Deserialize, Serialize};
use ulid::Ulid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    id: Ulid,
    username: String,
    native_language: NativeLanguage,
    current_japanese_level: JapaneseLevel,
    settings: UserSettings,
    knowledge_set: KnowledgeSet,
}

impl User {
    pub fn new(
        username: String,
        current_japanese_level: JapaneseLevel,
        native_language: NativeLanguage,
    ) -> Self {
        Self {
            id: Ulid::new(),
            username,
            knowledge_set: KnowledgeSet::new(),
            current_japanese_level,
            native_language,
            settings: UserSettings::empty(),
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

    pub fn native_language(&self) -> &NativeLanguage {
        &self.native_language
    }

    pub fn knowledge_set(&self) -> &KnowledgeSet {
        &self.knowledge_set
    }

    pub fn settings(&self) -> &UserSettings {
        &self.settings
    }

    pub fn settings_mut(&mut self) -> &mut UserSettings {
        &mut self.settings
    }

    pub fn rate_card(
        &mut self,
        card_id: Ulid,
        rating: Rating,
        interval: Duration,
        memory_state: MemoryState,
    ) -> Result<(), KeikakuError> {
        self.knowledge_set
            .rate_card(card_id, rating, interval, memory_state)?;
        Ok(())
    }

    pub fn add_lesson_duration(&mut self, lesson_duration: Duration) {
        self.knowledge_set.add_lesson_duration(lesson_duration);
    }

    pub fn delete_card(&mut self, card_id: Ulid) -> Result<(), KeikakuError> {
        self.knowledge_set.delete_card(card_id)
    }

    pub fn create_card(&mut self, card: Card) -> Result<StudyCard, KeikakuError> {
        self.knowledge_set.create_card(card)
    }
}
