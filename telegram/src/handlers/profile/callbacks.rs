use origa::domain::JapaneseLevel;
use serde::{Deserialize, Serialize};

/// Callback data types for profile module
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ProfileCallback {
    /// Open JLPT level selection
    #[serde(rename = "profile_jlpt")]
    JlptSelect,

    /// Set JLPT level
    #[serde(rename = "jlpt_set")]
    JlptSet { level: JapaneseLevel },

    /// Connect Duolingo
    #[serde(rename = "profile_duolingo")]
    DuolingoConnect,

    /// Open settings
    #[serde(rename = "profile_settings")]
    Settings,

    /// Toggle reminders
    #[serde(rename = "profile_reminders")]
    RemindersToggle,

    /// Exit profile
    #[serde(rename = "profile_exit")]
    Exit,

    /// Confirm exit
    #[serde(rename = "profile_confirm_exit")]
    ConfirmExit,

    /// Go back
    #[serde(rename = "profile_back")]
    Back,
}

impl ProfileCallback {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("Failed to serialize callback data")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_jlpt_select() {
        let callback = ProfileCallback::JlptSelect;
        let json = callback.to_json();
        assert!(json.contains(r#""kind":"profile_jlpt""#));
    }

    #[test]
    fn test_serialize_jlpt_set() {
        let callback = ProfileCallback::JlptSet {
            level: JapaneseLevel::N5,
        };
        let json = callback.to_json();
        assert!(json.contains(r#""kind":"jlpt_set""#));
        assert!(json.contains(r#""level":"N5""#));
    }
}
