use std::{collections::HashMap, sync::OnceLock};

use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::domain::{JapaneseLevel, NativeLanguage, OrigaError, PartOfSpeech};

pub static GRAMMAR_RULES: OnceLock<Vec<GrammarRule>> = OnceLock::new();

#[derive(Deserialize)]
struct GrammarStoreValue {
    grammar: Vec<GrammarRule>,
}

#[derive(Clone, Serialize, Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct GrammarData {
    pub grammar_json: String,
}

pub fn init_grammar(data: GrammarData) -> Result<(), OrigaError> {
    if is_grammar_loaded() {
        return Ok(());
    }

    let content: GrammarStoreValue =
        serde_json::from_str(&data.grammar_json).map_err(|e| OrigaError::GrammarParseError {
            reason: format!("Failed to parse grammar.json: {}", e),
        })?;

    GRAMMAR_RULES
        .set(content.grammar)
        .map_err(|_| OrigaError::GrammarParseError {
            reason: "Failed to set grammar rules".to_string(),
        })
}

/// Serialize GrammarData to rkyv bytes
pub fn serialize_grammar_to_rkyv(data: &GrammarData) -> Result<Vec<u8>, OrigaError> {
    let bytes =
        rkyv::to_bytes::<rkyv::rancor::Error>(data).map_err(|e| OrigaError::GrammarParseError {
            reason: format!("Failed to serialize grammar: {}", e),
        })?;
    Ok(bytes.to_vec())
}

/// Initialize grammar from rkyv bytes
pub fn init_grammar_from_rkyv(bytes: &[u8]) -> Result<(), OrigaError> {
    let archived =
        rkyv::access::<ArchivedGrammarData, rkyv::rancor::Error>(bytes).map_err(|e| {
            OrigaError::GrammarParseError {
                reason: format!("Failed to validate grammar data: {:?}", e),
            }
        })?;

    let data = GrammarData {
        grammar_json: archived.grammar_json.as_str().to_string(),
    };

    init_grammar(data)
}

pub fn is_grammar_loaded() -> bool {
    GRAMMAR_RULES.get().is_some()
}

pub fn get_rule_by_id(rule_id: &Ulid) -> Option<&'static GrammarRule> {
    GRAMMAR_RULES.get()?.iter().find(|x| x.rule_id() == rule_id)
}

pub fn get_rule_by_title(title: &str) -> Option<&'static GrammarRule> {
    GRAMMAR_RULES
        .get()?
        .iter()
        .find(|x| x.content.values().any(|c| c.title() == title))
}

pub fn iter_grammar_rules() -> impl Iterator<Item = &'static GrammarRule> {
    GRAMMAR_RULES
        .get()
        .into_iter()
        .flat_map(|rules| rules.iter())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrammarRule {
    rule_id: Ulid,
    level: JapaneseLevel,
    content: HashMap<NativeLanguage, GrammarRuleContent>,
    format_map: Option<HashMap<PartOfSpeech, Vec<FormatAction>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrammarRuleContent {
    title: String,
    short_description: String,
    md_description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FormatAction {
    AdjectiveRemovePostfix {},
    AdjectiveToKunai {},
    AdjectiveToKatta {},
    AdjectiveToKunakatta {},
    AdjectiveToKute {},
    AdjectiveToKu {},
    AdjectiveToKereba {},
    AdjectiveToSou {},
    AdjectiveToSugiru {},
    AdjectiveToNa {},
    AdjectiveToDe {},
    AdjectiveToNara {},
    AdjectiveToSouNa {},
    AdjectiveToNasasou {},
    AdjectiveToGaru {},

    VerbToTeForm {},
    VerbToMainView {},
    VerbToMasu {},
    VerbToMasen {},
    VerbToMashita {},
    VerbToMasenDeshita {},
    VerbToMashou {},
    VerbToStem {},
    VerbToTa {},
    VerbToNai {},
    VerbToTara {},
    VerbToBa {},
    VerbToPotential {},
    VerbToPassive {},
    VerbToCausative {},
    VerbToCausativePassive {},
    VerbToImperative {},
    VerbToVolitional {},
    VerbToSou {},
    VerbToZu {},
    VerbToTai {},
    VerbToYasui {},
    VerbToNikui {},
    VerbToSugiru {},
    VerbToChau {},
    VerbToToku {},
    VerbToTeru {},
    VerbToONinarimasu {},
    VerbToOKudasai {},
    VerbToOShimasu {},

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
    #[cfg(test)]
    pub fn new(
        rule_id: Ulid,
        level: JapaneseLevel,
        content: HashMap<NativeLanguage, GrammarRuleContent>,
        format_map: Option<HashMap<PartOfSpeech, Vec<FormatAction>>>,
    ) -> Self {
        Self {
            rule_id,
            level,
            content,
            format_map,
        }
    }

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
        match &self.format_map {
            Some(map) => map.keys().cloned().collect(),
            None => vec![],
        }
    }

    pub(crate) fn format_map(&self) -> Option<&HashMap<PartOfSpeech, Vec<FormatAction>>> {
        self.format_map.as_ref()
    }

    pub fn has_format_map(&self) -> bool {
        self.format_map.is_some()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grammar_rules_should_not_be_loaded_before_init() {
        assert!(!is_grammar_loaded());
    }
}
