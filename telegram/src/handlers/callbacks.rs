use serde::{Deserialize, Serialize};

use crate::handlers::add_from_text::callbacks::AddFromTextCallback;
use crate::handlers::grammar::GrammarCallback;
use crate::handlers::kanji::KanjiCallback;
use crate::handlers::lesson::LessonCallback;
use crate::handlers::menu::MenuCallback;
use crate::handlers::profile::ProfileCallback;
use crate::handlers::vocabulary::VocabularyCallback;

/// Main callback data enum for all modules
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CallbackData {
    #[serde(rename = "add_from_text")]
    AddFromText(AddFromTextCallback),

    #[serde(rename = "grammar")]
    Grammar(GrammarCallback),

    #[serde(rename = "lesson")]
    Lesson(LessonCallback),

    #[serde(rename = "profile")]
    Profile(ProfileCallback),

    #[serde(rename = "vocabulary")]
    Vocabulary(VocabularyCallback),

    #[serde(rename = "kanji")]
    Kanji(KanjiCallback),

    #[serde(rename = "menu")]
    Menu(MenuCallback),
}

impl CallbackData {
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    pub fn try_from_json(json: &str) -> Option<Self> {
        Self::from_json(json).ok()
    }
}
