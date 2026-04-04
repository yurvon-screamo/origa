mod error;
mod furigana;
pub(crate) mod grammar;
mod japanese;
mod jlpt_content;
mod jlpt_progress;
mod knowledge;
mod memory;
mod score_content;
mod srs;
mod tokenizer;
mod user;
pub(crate) mod value_objects;

pub use crate::dictionary::grammar::{GrammarRule, get_rule_by_id, iter_grammar_rules};
pub use crate::dictionary::vocabulary::get_translation;
pub use error::OrigaError;
pub use furigana::{FuriganaSegment, furiganize_segments, furiganize_text, furiganize_text_html};
pub use japanese::{JapaneseChar, JapaneseText, filter_japanese_text};
pub use jlpt_content::{JlptContent, JlptContentError};
pub use jlpt_progress::{CategoryProgress, JlptProgress, LevelProgressDetail};
pub use knowledge::{
    Card, CardType, DailyHistoryItem, ExampleKanjiWord, GrammarInfo, GrammarRuleCard, KanjiCard,
    KnowledgeSet, LessonCardView, LessonViewGenerator, QuizCard, QuizOption, StudyCard,
    VocabularyCard, YesNoCard,
};
pub use memory::{Difficulty, MemoryHistory, MemoryState, Rating, ReviewLog, Stability};
pub use score_content::ScoreContentResult;
pub use srs::RateMode;
pub use tokenizer::{
    DictionaryData, PartOfSpeech, TokenInfo, init_dictionary, init_dictionary_from_rkyv,
    is_dictionary_loaded, serialize_dictionary_to_rkyv, tokenize_text,
};
pub use user::{User, WordKnowledge};
pub use value_objects::{Answer, DailyLoad, JapaneseLevel, NativeLanguage, Question};
