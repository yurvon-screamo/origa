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
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("Failed to serialize callback data")
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    pub fn try_from_json(json: &str) -> Option<Self> {
        Self::from_json(json).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_page() {
        let callback = VocabularyCallback::Page { page: 1 };
        let json = callback.to_json();
        assert!(json.contains(r#""kind":"vocab_page""#));
        assert!(json.contains(r#""page":1"#));
    }

    #[test]
    fn test_serialize_detail() {
        let card_id = Ulid::new();
        let callback = VocabularyCallback::Detail { card_id };
        let json = callback.to_json();
        assert!(json.contains(r#""kind":"vocab_detail""#));
        assert!(json.contains(r#""card_id":"#));
    }

    #[test]
    fn test_serialize_filter() {
        let callback = VocabularyCallback::Filter {
            filter: "new".to_string(),
        };
        let json = callback.to_json();
        assert!(json.contains(r#""kind":"vocab_filter""#));
        assert!(json.contains(r#""filter":"new""#));
    }

    #[test]
    fn test_deserialize_page() {
        let json = r#"{"kind":"vocab_page","page":2}"#;
        let callback = VocabularyCallback::from_json(json).unwrap();
        assert_eq!(callback, VocabularyCallback::Page { page: 2 });
    }

    #[test]
    fn test_serialize_deserialize_roundtrip() {
        let original = VocabularyCallback::Add {
            card_id: Ulid::new(),
        };
        let json = original.to_json();
        let deserialized = VocabularyCallback::from_json(&json).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_all_variants_serializable() {
        let card_id = Ulid::new();
        let variants = vec![
            VocabularyCallback::Filter {
                filter: "all".to_string(),
            },
            VocabularyCallback::Page { page: 0 },
            VocabularyCallback::PageCurrent,
            VocabularyCallback::Detail { card_id },
            VocabularyCallback::Add { card_id },
            VocabularyCallback::Delete { card_id },
            VocabularyCallback::ConfirmDelete { card_id },
            VocabularyCallback::CancelDelete,
            VocabularyCallback::AddFromText,
            VocabularyCallback::Search,
            VocabularyCallback::SearchPage {
                page: 0,
                query: "test".to_string(),
            },
            VocabularyCallback::SearchCurrent,
            VocabularyCallback::BackToList,
            VocabularyCallback::MainMenu,
        ];

        for variant in variants {
            let json = variant.to_json();
            let deserialized = VocabularyCallback::from_json(&json).unwrap();
            assert_eq!(variant, deserialized);
        }
    }

    #[test]
    fn test_try_from_json_success() {
        let json = r#"{"kind":"vocab_detail","card_id":"01ARZ3NDEKTSV4RRFFQ69G5FAV"}"#;
        let callback = VocabularyCallback::try_from_json(json);
        assert!(callback.is_some());
    }

    #[test]
    fn test_try_from_json_fail() {
        let json = r#"{"kind":"unknown","card_id":"01ARZ3NDEKTSV4RRFFQ69G5FAV"}"#;
        let callback = VocabularyCallback::try_from_json(json);
        assert!(callback.is_none());
    }

    #[test]
    fn test_search_page_with_query() {
        let callback = VocabularyCallback::SearchPage {
            page: 1,
            query: "日本語".to_string(),
        };
        let json = callback.to_json();
        assert!(json.contains(r#""kind":"vocab_search_page""#));
        assert!(json.contains(r#""page":1"#));
        assert!(json.contains(r#""query":"日本語""#));
    }
}
