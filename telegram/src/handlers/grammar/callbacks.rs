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
    /// Serialize callback data to JSON string for use in callback_data
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("Failed to serialize callback data")
    }

    /// Deserialize callback data from JSON string
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Try to parse callback data, returns None if parsing fails
    pub fn try_from_json(json: &str) -> Option<Self> {
        Self::from_json(json).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_page() {
        let callback = GrammarCallback::Page { page: 1 };
        let json = callback.to_json();
        assert!(json.contains(r#""kind":"grammar_page""#));
        assert!(json.contains(r#""page":1"#));
    }

    #[test]
    fn test_serialize_detail() {
        let rule_id = Ulid::new();
        let callback = GrammarCallback::Detail { rule_id };
        let json = callback.to_json();
        assert!(json.contains(r#""kind":"grammar_detail""#));
        assert!(json.contains(r#""rule_id":"#));
    }

    #[test]
    fn test_deserialize_page() {
        let json = r#"{"kind":"grammar_page","page":2}"#;
        let callback = GrammarCallback::from_json(json).unwrap();
        assert_eq!(callback, GrammarCallback::Page { page: 2 });
    }

    #[test]
    fn test_serialize_deserialize_roundtrip() {
        let original = GrammarCallback::Add {
            rule_id: Ulid::new(),
        };
        let json = original.to_json();
        let deserialized = GrammarCallback::from_json(&json).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_all_variants_serializable() {
        let rule_id = Ulid::new();
        let variants = vec![
            GrammarCallback::Page { page: 0 },
            GrammarCallback::Detail { rule_id },
            GrammarCallback::Add { rule_id },
            GrammarCallback::Delete { rule_id },
            GrammarCallback::BackToList,
            GrammarCallback::CurrentPage,
            GrammarCallback::Search,
        ];

        for variant in variants {
            let json = variant.to_json();
            let deserialized = GrammarCallback::from_json(&json).unwrap();
            assert_eq!(variant, deserialized);
        }
    }
}
