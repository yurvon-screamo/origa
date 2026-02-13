use serde::{Deserialize, Serialize};
use ulid::Ulid;

/// Callback data types for vocabulary module
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum VocabularyCallback {
    /// Filter vocabulary list by status
    #[serde(rename = "vocab_filter")]
    Filter { filter: String },

    /// Navigate to a specific page of vocabulary list
    #[serde(rename = "vocab_page")]
    Page { page: usize },

    /// Current page indicator (no action)
    #[serde(rename = "vocab_page_current")]
    PageCurrent,

    /// Show details of a specific vocabulary card
    #[serde(rename = "vocab_detail")]
    Detail { card_id: Ulid },

    /// Add a vocabulary card to user's set
    #[serde(rename = "vocab_add")]
    Add { card_id: Ulid },

    /// Request deletion of a vocabulary card
    #[serde(rename = "vocab_delete")]
    Delete { card_id: Ulid },

    /// Confirm deletion of a vocabulary card
    #[serde(rename = "vocab_confirm_delete")]
    ConfirmDelete { card_id: Ulid },

    /// Cancel deletion request
    #[serde(rename = "vocab_cancel_delete")]
    CancelDelete,

    /// Add vocabulary cards from text
    #[serde(rename = "vocab_add_from_text")]
    AddFromText,

    /// Search vocabulary with query
    #[serde(rename = "vocab_search")]
    Search,

    /// Navigate to a specific page of search results
    #[serde(rename = "vocab_search_page")]
    SearchPage { page: usize, query: String },

    /// Current search page indicator (no action)
    #[serde(rename = "vocab_search_current")]
    SearchCurrent,

    /// Navigate back to the vocabulary list
    #[serde(rename = "vocab_back_to_list")]
    BackToList,

    /// Navigate to main menu
    #[serde(rename = "menu_home")]
    MainMenu,
}

impl VocabularyCallback {
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    pub fn try_from_json(json: &str) -> Option<Self> {
        Self::from_json(json).ok()
    }
}
