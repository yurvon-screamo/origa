use std::sync::OnceLock;

use serde::Deserialize;
use ulid::Ulid;

use crate::domain::{GrammarRule, OrigaError};

pub static GRAMMAR_RULES: OnceLock<Vec<GrammarRule>> = OnceLock::new();

#[derive(Deserialize)]
struct GrammarStoreValue {
    grammar: Vec<GrammarRule>,
}

pub struct GrammarData {
    pub grammar_json: String,
}

pub fn init_grammar_rules(data: GrammarData) -> Result<(), OrigaError> {
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

pub fn is_grammar_loaded() -> bool {
    GRAMMAR_RULES.get().is_some()
}

pub fn get_rule_by_id(rule_id: &Ulid) -> Option<&'static GrammarRule> {
    GRAMMAR_RULES.get()?.iter().find(|x| x.rule_id() == rule_id)
}

pub fn iter_grammar_rules() -> impl Iterator<Item = &'static GrammarRule> {
    GRAMMAR_RULES
        .get()
        .into_iter()
        .flat_map(|rules| rules.iter())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grammar_rules_should_not_be_loaded_before_init() {
        assert!(!is_grammar_loaded());
    }
}
