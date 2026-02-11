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
        let callback = KanjiCallback::Page { page: 1 };
        let json = callback.to_json();
        assert!(json.contains(r#""kind":"kanji_page""#));
        assert!(json.contains(r#""page":1"#));
    }

    #[test]
    fn test_serialize_detail() {
        let callback = KanjiCallback::Detail {
            kanji: "日".to_string(),
        };
        let json = callback.to_json();
        assert!(json.contains(r#""kind":"kanji_detail""#));
        assert!(json.contains(r#""kanji":"日""#));
    }

    #[test]
    fn test_deserialize_page() {
        let json = r#"{"kind":"kanji_page","page":2}"#;
        let callback = KanjiCallback::from_json(json).unwrap();
        assert_eq!(callback, KanjiCallback::Page { page: 2 });
    }

    #[test]
    fn test_serialize_deserialize_roundtrip() {
        let original = KanjiCallback::Add {
            kanji: "本".to_string(),
        };
        let json = original.to_json();
        let deserialized = KanjiCallback::from_json(&json).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_all_variants_serializable() {
        let variants = vec![
            KanjiCallback::Level {
                level: JapaneseLevel::N5,
            },
            KanjiCallback::Page { page: 0 },
            KanjiCallback::PageCurrent,
            KanjiCallback::Detail {
                kanji: "日".to_string(),
            },
            KanjiCallback::Add {
                kanji: "本".to_string(),
            },
            KanjiCallback::Delete {
                kanji: "月".to_string(),
            },
            KanjiCallback::AddNew,
            KanjiCallback::Search {
                query: "test".to_string(),
                page: 0,
            },
            KanjiCallback::BackToList,
            KanjiCallback::MainMenu,
        ];

        for variant in variants {
            let json = variant.to_json();
            let deserialized = KanjiCallback::from_json(&json).unwrap();
            assert_eq!(variant, deserialized);
        }
    }

    #[test]
    fn test_try_from_json_success() {
        let json = r#"{"kind":"kanji_add","kanji":"字"}"#;
        let callback = KanjiCallback::try_from_json(json);
        assert_eq!(
            callback,
            Some(KanjiCallback::Add {
                kanji: "字".to_string(),
            })
        );
    }

    #[test]
    fn test_try_from_json_fail() {
        let json = r#"{"kind":"unknown","kanji":"字"}"#;
        let callback = KanjiCallback::try_from_json(json);
        assert!(callback.is_none());
    }
}
