use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MenuCallback {
    #[serde(rename = "menu_home")]
    MainMenu,

    #[serde(rename = "menu_lesson")]
    Lesson,

    #[serde(rename = "menu_fixation")]
    Fixation,

    #[serde(rename = "menu_vocabulary")]
    Vocabulary,

    #[serde(rename = "menu_kanji")]
    Kanji,

    #[serde(rename = "menu_grammar")]
    Grammar,

    #[serde(rename = "menu_profile")]
    Profile,

    #[serde(rename = "menu_settings")]
    Settings,

    #[serde(rename = "history_known")]
    HistoryKnown,

    #[serde(rename = "history_in_progress")]
    HistoryInProgress,

    #[serde(rename = "history_new")]
    HistoryNew,

    #[serde(rename = "history_hard")]
    HistoryHard,

    #[serde(rename = "show_history")]
    ShowHistory,
}

impl MenuCallback {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("Failed to serialize callback data")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_main_menu() {
        let callback = MenuCallback::MainMenu;
        let json = callback.to_json();
        assert!(json.contains(r#""kind":"menu_home""#));
    }

    #[test]
    fn test_serialize_history_known() {
        let callback = MenuCallback::HistoryKnown;
        let json = callback.to_json();
        assert!(json.contains(r#""kind":"history_known""#));
    }
}
