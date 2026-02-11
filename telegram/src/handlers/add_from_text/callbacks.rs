use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AddFromTextCallback {
    #[serde(rename = "text_toggle")]
    Toggle { word: String },

    #[serde(rename = "text_confirm")]
    Confirm,

    #[serde(rename = "text_cancel")]
    Cancel,
}

impl AddFromTextCallback {
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
    fn test_serialize_toggle() {
        let callback = AddFromTextCallback::Toggle {
            word: "日本語".to_string(),
        };
        let json = callback.to_json();
        assert!(json.contains(r#""kind":"text_toggle""#));
        assert!(json.contains(r#""word":"日本語""#));
    }

    #[test]
    fn test_serialize_confirm() {
        let callback = AddFromTextCallback::Confirm;
        let json = callback.to_json();
        assert!(json.contains(r#""kind":"text_confirm""#));
    }

    #[test]
    fn test_serialize_cancel() {
        let callback = AddFromTextCallback::Cancel;
        let json = callback.to_json();
        assert!(json.contains(r#""kind":"text_cancel""#));
    }

    #[test]
    fn test_deserialize_toggle() {
        let json = r#"{"kind":"text_toggle","word":"test"}"#;
        let callback = AddFromTextCallback::from_json(json).unwrap();
        assert_eq!(
            callback,
            AddFromTextCallback::Toggle {
                word: "test".to_string()
            }
        );
    }

    #[test]
    fn test_serialize_deserialize_roundtrip() {
        let original = AddFromTextCallback::Toggle {
            word: "test".to_string(),
        };
        let json = original.to_json();
        let deserialized = AddFromTextCallback::from_json(&json).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_all_variants_serializable() {
        let variants = vec![
            AddFromTextCallback::Toggle {
                word: "word".to_string(),
            },
            AddFromTextCallback::Confirm,
            AddFromTextCallback::Cancel,
        ];

        for variant in variants {
            let json = variant.to_json();
            let deserialized = AddFromTextCallback::from_json(&json).unwrap();
            assert_eq!(variant, deserialized);
        }
    }

    #[test]
    fn test_try_from_json_success() {
        let json = r#"{"kind":"text_toggle","word":"test"}"#;
        let callback = AddFromTextCallback::try_from_json(json);
        assert!(callback.is_some());
    }

    #[test]
    fn test_try_from_json_fail() {
        let json = r#"{"kind":"unknown"}"#;
        let callback = AddFromTextCallback::try_from_json(json);
        assert!(callback.is_none());
    }
}
