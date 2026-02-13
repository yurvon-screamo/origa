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
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    pub fn try_from_json(json: &str) -> Option<Self> {
        Self::from_json(json).ok()
    }
}
