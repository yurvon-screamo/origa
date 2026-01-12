use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::domain::{
    OrigaError,
    grammar::{GrammarRule, GrammarRuleContent, GrammarRuleInfo, verb_forms::to_mashou_form},
    tokenizer::PartOfSpeech,
    value_objects::{JapaneseLevel, NativeLanguage},
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VerbMashouRule {
    rule: GrammarRuleInfo,
}

impl Default for VerbMashouRule {
    fn default() -> Self {
        Self::new()
    }
}

impl VerbMashouRule {
    pub fn new() -> Self {
        let mut content = HashMap::new();
        content.insert(
            NativeLanguage::Russian,
            GrammarRuleContent {
                title: "Форма ～ましょう".to_string(),
                short_description: "Давай сделаем".to_string(),
                md_description: "".to_string(),
            },
        );
        content.insert(
            NativeLanguage::English,
            GrammarRuleContent {
                title: "Form ～ましょう".to_string(),
                short_description: "".to_string(),
                md_description: "".to_string(),
            },
        );

        let rule = GrammarRuleInfo::new(
            Ulid::from_string("01D39ZY06FGSCTVN4T2V9PKHFA").expect("Invalid ID"),
            JapaneseLevel::N5,
            vec![PartOfSpeech::Verb],
            content,
        );

        Self { rule }
    }
}

impl GrammarRule for VerbMashouRule {
    fn format(&self, word: &str, part_of_speech: &PartOfSpeech) -> Result<String, OrigaError> {
        match part_of_speech {
            PartOfSpeech::Verb => Ok(to_mashou_form(word)),
            _ => Err(OrigaError::GrammarFormatError {
                reason: "Not supported part of speech".to_string(),
            }),
        }
    }

    fn info(&self) -> &GrammarRuleInfo {
        &self.rule
    }
}
