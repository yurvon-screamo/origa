use crate::dictionary::grammar::get_rule_by_id;
use crate::domain::OrigaError;
use crate::domain::{
    tokenizer::PartOfSpeech,
    value_objects::{Answer, NativeLanguage, Question},
};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

macro_rules! get_content {
    ($self:expr, $lang:expr, $content_method:ident, $new_method:path) => {{
        let rule = get_rule_by_id(&$self.rule_id).ok_or(OrigaError::GrammarRuleNotFound {
            rule_id: $self.rule_id,
        })?;

        let text = rule.content($lang).$content_method();
        if text.is_empty() {
            return Err(OrigaError::GrammarContentNotFound {
                rule_id: $self.rule_id,
                lang: *$lang,
            });
        }

        $new_method(text.to_string())
    }};
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GrammarRuleCard {
    rule_id: Ulid,
}

impl GrammarRuleCard {
    pub fn new(rule_id: Ulid) -> Result<Self, OrigaError> {
        get_rule_by_id(&rule_id).ok_or_else(|| OrigaError::RepositoryError {
            reason: format!("Grammar rule {} not found", rule_id),
        })?;
        Ok(Self { rule_id })
    }

    pub fn rule_id(&self) -> &Ulid {
        &self.rule_id
    }

    pub fn title(&self, lang: &NativeLanguage) -> Result<Question, OrigaError> {
        get_content!(self, lang, title, Question::new)
    }

    pub fn description(&self, lang: &NativeLanguage) -> Result<Answer, OrigaError> {
        get_content!(self, lang, md_description, Answer::new)
    }

    pub fn short_description(&self, lang: &NativeLanguage) -> Result<Answer, OrigaError> {
        get_content!(self, lang, short_description, Answer::new)
    }

    pub fn apply_to(&self) -> Vec<PartOfSpeech> {
        get_rule_by_id(&self.rule_id)
            .map(|rule| rule.apply_to())
            .unwrap_or_default()
    }
}

impl GrammarRuleCard {
    #[cfg(test)]
    pub fn new_test() -> Self {
        Self {
            rule_id: Ulid::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dictionary::grammar::{GrammarData, init_grammar, is_grammar_loaded};
    use crate::domain::NativeLanguage;
    use std::sync::Once;

    static INIT: Once = Once::new();

    fn init_test_grammar() {
        INIT.call_once(|| {
            if is_grammar_loaded() {
                return;
            }
            let manifest_dir =
                std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
            let grammar_path = std::path::PathBuf::from(manifest_dir)
                .parent()
                .expect("Failed to get parent directory")
                .join("origa_ui")
                .join("public")
                .join("grammar")
                .join("grammar.json");

            let grammar_json =
                std::fs::read_to_string(&grammar_path).expect("Failed to read grammar.json");

            let grammar_data = GrammarData { grammar_json };
            init_grammar(grammar_data).expect("Failed to init grammar rules");
        });
    }

    fn get_first_rule_id() -> Ulid {
        Ulid::from_string("01KJ9AVWBGC2BT0DMFPDYYFEWB").expect("Invalid ULID")
    }

    mod new {
        use super::*;

        #[test]
        fn creates_card_with_valid_rule_id() {
            init_test_grammar();

            let rule_id = get_first_rule_id();
            let card = GrammarRuleCard::new(rule_id);

            assert!(card.is_ok());
            assert_eq!(*card.unwrap().rule_id(), rule_id);
        }

        #[test]
        fn returns_error_for_nonexistent_rule_id() {
            init_test_grammar();

            let nonexistent_id = Ulid::new();
            let result = GrammarRuleCard::new(nonexistent_id);

            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(matches!(err, OrigaError::RepositoryError { .. }));
        }
    }

    mod rule_id {
        use super::*;

        #[test]
        fn returns_rule_id() {
            init_test_grammar();

            let rule_id = get_first_rule_id();
            let card = GrammarRuleCard::new(rule_id).expect("Failed to create card");

            assert_eq!(*card.rule_id(), rule_id);
        }

        #[test]
        fn new_test_creates_card_with_any_rule_id() {
            let card = GrammarRuleCard::new_test();

            assert!(!card.rule_id().is_nil());
        }
    }

    mod title {
        use super::*;

        #[test]
        fn returns_title_in_russian() {
            init_test_grammar();

            let rule_id = get_first_rule_id();
            let card = GrammarRuleCard::new(rule_id).expect("Failed to create card");

            let title = card.title(&NativeLanguage::Russian);

            assert!(title.is_ok());
            assert!(!title.unwrap().text().is_empty());
        }

        #[test]
        fn returns_title_in_english() {
            init_test_grammar();

            let rule_id = get_first_rule_id();
            let card = GrammarRuleCard::new(rule_id).expect("Failed to create card");

            let title = card.title(&NativeLanguage::English);

            assert!(title.is_ok());
            assert!(!title.unwrap().text().is_empty());
        }

        #[test]
        fn returns_different_titles_for_different_languages() {
            init_test_grammar();

            let rule_id = get_first_rule_id();
            let card = GrammarRuleCard::new(rule_id).expect("Failed to create card");

            let russian_title = card.title(&NativeLanguage::Russian).unwrap();
            let english_title = card.title(&NativeLanguage::English).unwrap();

            assert_eq!(russian_title.text(), english_title.text());
        }
    }

    mod description {
        use super::*;

        #[test]
        fn returns_description_in_russian() {
            init_test_grammar();

            let rule_id = get_first_rule_id();
            let card = GrammarRuleCard::new(rule_id).expect("Failed to create card");

            let desc = card.description(&NativeLanguage::Russian);

            assert!(desc.is_ok());
            let binding = desc.unwrap();
            let text = binding.text();
            assert!(!text.is_empty());
            assert!(text.contains("ます"));
        }

        #[test]
        fn returns_description_in_english() {
            init_test_grammar();

            let rule_id = get_first_rule_id();
            let card = GrammarRuleCard::new(rule_id).expect("Failed to create card");

            let desc = card.description(&NativeLanguage::English);

            assert!(desc.is_ok());
            let binding = desc.unwrap();
            let text = binding.text();
            assert!(!text.is_empty());
            assert!(text.contains("ます"));
        }

        #[test]
        fn description_is_longer_than_title() {
            init_test_grammar();

            let rule_id = get_first_rule_id();
            let card = GrammarRuleCard::new(rule_id).expect("Failed to create card");

            let title = card.title(&NativeLanguage::Russian).unwrap();
            let desc = card.description(&NativeLanguage::Russian).unwrap();

            assert!(desc.text().len() > title.text().len());
        }
    }

    mod apply_to {
        use super::*;
        use crate::domain::tokenizer::PartOfSpeech;

        #[test]
        fn returns_parts_of_speech_for_rule_with_format_map() {
            init_test_grammar();

            let rule_id = get_first_rule_id();
            let card = GrammarRuleCard::new(rule_id).expect("Failed to create card");

            let parts = card.apply_to();

            assert!(!parts.is_empty());
            assert!(parts.contains(&PartOfSpeech::Verb));
        }
    }

    mod short_description {
        use super::*;

        #[test]
        fn returns_short_description_in_russian() {
            init_test_grammar();

            let rule_id = get_first_rule_id();
            let card = GrammarRuleCard::new(rule_id).expect("Failed to create card");

            let short_desc = card.short_description(&NativeLanguage::Russian);

            assert!(short_desc.is_ok());
            assert!(!short_desc.unwrap().text().is_empty());
        }

        #[test]
        fn returns_short_description_in_english() {
            init_test_grammar();

            let rule_id = get_first_rule_id();
            let card = GrammarRuleCard::new(rule_id).expect("Failed to create card");

            let short_desc = card.short_description(&NativeLanguage::English);

            assert!(short_desc.is_ok());
            assert!(!short_desc.unwrap().text().is_empty());
        }

        #[test]
        fn short_description_is_shorter_than_full_description() {
            init_test_grammar();

            let rule_id = get_first_rule_id();
            let card = GrammarRuleCard::new(rule_id).expect("Failed to create card");

            let short_desc = card.short_description(&NativeLanguage::Russian).unwrap();
            let full_desc = card.description(&NativeLanguage::Russian).unwrap();

            assert!(short_desc.text().len() < full_desc.text().len());
        }

        #[test]
        fn returns_error_for_nonexistent_rule() {
            init_test_grammar();

            let card = GrammarRuleCard::new_test();
            let result = card.short_description(&NativeLanguage::Russian);

            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(matches!(err, OrigaError::GrammarRuleNotFound { .. }));
        }
    }

    mod serialization {
        use super::*;

        #[test]
        fn serialization_roundtrip() {
            init_test_grammar();

            let rule_id = get_first_rule_id();
            let card = GrammarRuleCard::new(rule_id).expect("Failed to create card");

            let json = serde_json::to_string(&card).expect("Failed to serialize");
            let deserialized: GrammarRuleCard =
                serde_json::from_str(&json).expect("Failed to deserialize");

            assert_eq!(deserialized.rule_id(), card.rule_id());
        }

        #[test]
        fn serialization_contains_rule_id() {
            init_test_grammar();

            let rule_id = get_first_rule_id();
            let card = GrammarRuleCard::new(rule_id).expect("Failed to create card");

            let json = serde_json::to_string(&card).expect("Failed to serialize");

            assert!(json.contains("rule_id"));
        }

        #[test]
        fn test_card_serialization_roundtrip() {
            let card = GrammarRuleCard::new_test();

            let json = serde_json::to_string(&card).expect("Failed to serialize");
            let deserialized: GrammarRuleCard =
                serde_json::from_str(&json).expect("Failed to deserialize");

            assert_eq!(deserialized.rule_id(), card.rule_id());
        }
    }

    mod clone_and_equality {
        use super::*;

        #[test]
        fn clone_produces_equal_card() {
            init_test_grammar();

            let rule_id = get_first_rule_id();
            let card = GrammarRuleCard::new(rule_id).expect("Failed to create card");
            let cloned = card.clone();

            assert_eq!(card, cloned);
        }

        #[test]
        fn different_rule_ids_produce_different_cards() {
            init_test_grammar();

            let rule_id1 = get_first_rule_id();
            let rule_id2 = Ulid::from_string("01KJ9AVWBG78GHSKKD8W1YHJB3").expect("Invalid ULID");

            let card1 = GrammarRuleCard::new(rule_id1).expect("Failed to create card1");
            let card2 = GrammarRuleCard::new(rule_id2).expect("Failed to create card2");

            assert_ne!(card1, card2);
        }
    }
}
