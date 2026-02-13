use origa::domain::JapaneseLevel;
use serde::{Deserialize, Serialize};

/// Callback data types for kanji module
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum KanjiCallback {
    /// Filter kanji list by level (N5, N4, N3, N2, N1, all)
    #[serde(rename = "kanji_level")]
    Level { level: JapaneseLevel },

    /// Navigate to a specific page of kanji list
    #[serde(rename = "kanji_page")]
    Page { page: usize },

    /// Current page indicator (no action)
    #[serde(rename = "kanji_current_page")]
    PageCurrent,

    /// Show details of a specific kanji
    #[serde(rename = "kanji_detail")]
    Detail { kanji: String },

    /// Add a specific kanji to user's set
    #[serde(rename = "kanji_add")]
    Add { kanji: String },

    /// Delete a specific kanji from user's set
    #[serde(rename = "kanji_delete")]
    Delete { kanji: String },

    /// Add new kanji from list
    #[serde(rename = "kanji_add_new")]
    AddNew,

    /// Search kanji with query and pagination
    #[serde(rename = "kanji_search")]
    Search { query: String, page: usize },

    /// Navigate back to the kanji list
    #[serde(rename = "kanji_back_to_list")]
    BackToList,

    /// Navigate to main menu
    #[serde(rename = "menu_home")]
    MainMenu,
}

impl KanjiCallback {
    /// Deserialize callback data from JSON string
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Try to parse callback data, returns None if parsing fails
    pub fn try_from_json(json: &str) -> Option<Self> {
        Self::from_json(json).ok()
    }
}
