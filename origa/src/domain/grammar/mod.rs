mod forms_adjective;
mod forms_verb;
mod store;

pub use store::{GRAMMAR_RULES, get_rule_by_id};

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::domain::{
    OrigaError,
    grammar::{
        forms_adjective::adjective_remove_postfix,
        forms_verb::{to_main_view, to_te_form},
    },
    tokenizer::PartOfSpeech,
    value_objects::{JapaneseLevel, NativeLanguage},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrammarRule {
    rule_id: Ulid,
    level: JapaneseLevel,
    content: HashMap<NativeLanguage, GrammarRuleContent>,
    format_map: HashMap<PartOfSpeech, Vec<FormatAction>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrammarRuleContent {
    title: String,
    short_description: String,
    md_description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FormatAction {
    AdjectiveRemovePostfix,

    VerbToTeForm,
    VerbToMainView,

    ReplacePostfix {
        old_postfix: String,
        new_postfix: String,
    },
    AddPostfix {
        postfix: String,
    },
    RemovePostfix {
        postfix: String,
    },
}

impl GrammarRule {
    pub fn rule_id(&self) -> &Ulid {
        &self.rule_id
    }

    pub fn level(&self) -> &JapaneseLevel {
        &self.level
    }

    pub fn content(&self, lang: &NativeLanguage) -> &GrammarRuleContent {
        &self.content[lang]
    }

    pub fn apply_to(&self) -> Vec<PartOfSpeech> {
        self.format_map.keys().cloned().collect()
    }

    pub fn format(
        &self,
        source_word: &str,
        part_of_speech: &PartOfSpeech,
    ) -> Result<String, OrigaError> {
        let rules = self
            .format_map
            .get(part_of_speech)
            .ok_or(OrigaError::GrammarFormatError {
                reason: "Not supported part of speech".to_string(),
            })?;

        let result = rules
            .iter()
            .try_fold(source_word.to_string(), |word, rule| match rule {
                FormatAction::AdjectiveRemovePostfix => {
                    adjective_remove_postfix(&word, part_of_speech)
                }
                FormatAction::VerbToTeForm => Ok(to_te_form(&word)),
                FormatAction::VerbToMainView => Ok(to_main_view(&word)),
                FormatAction::AddPostfix { postfix } => Ok(word + postfix),
                FormatAction::ReplacePostfix {
                    old_postfix,
                    new_postfix,
                } => Ok(word.trim_end_matches(old_postfix).to_string() + new_postfix),
                FormatAction::RemovePostfix { postfix } => {
                    Ok(word.trim_end_matches(postfix).to_string())
                }
            })?;

        Ok(result)
    }
}

impl GrammarRuleContent {
    pub fn new(title: String, short_description: String, md_description: String) -> Self {
        Self {
            title,
            short_description,
            md_description,
        }
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn short_description(&self) -> &str {
        &self.short_description
    }

    pub fn md_description(&self) -> &str {
        &self.md_description
    }
}
