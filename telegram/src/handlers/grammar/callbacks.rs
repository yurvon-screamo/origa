use serde::{Deserialize, Serialize};
use ulid::Ulid;

/// Callback data types for grammar module
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum GrammarCallback {
    /// Navigate to a specific page of grammar rules
    #[serde(rename = "grammar_page")]
    Page { page: usize },

    /// Show details of a specific grammar rule
    #[serde(rename = "grammar_detail")]
    Detail { rule_id: Ulid },

    /// Add a grammar rule to user's set
    #[serde(rename = "grammar_add")]
    Add { rule_id: Ulid },

    /// Delete a grammar rule from user's set
    #[serde(rename = "grammar_delete")]
    Delete { rule_id: Ulid },

    /// Navigate back to the grammar list
    #[serde(rename = "grammar_back_to_list")]
    BackToList,

    /// Current page indicator (no action)
    #[serde(rename = "grammar_current_page")]
    CurrentPage,

    /// Search grammar rules
    #[serde(rename = "grammar_search")]
    Search,
}

impl GrammarCallback {
    /// Deserialize callback data from JSON string
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Try to parse callback data, returns None if parsing fails
    pub fn try_from_json(json: &str) -> Option<Self> {
        Self::from_json(json).ok()
    }
}
