use std::sync::LazyLock;

use serde::Deserialize;
use tracing::error;
use ulid::Ulid;

use crate::domain::GrammarRule;

const GRAMMAR_DATA: &str = include_str!("./grammar.json");

#[derive(Deserialize)]
struct GrammarStoreValue {
    grammar: Vec<GrammarRule>,
}

pub static GRAMMAR_RULES: LazyLock<Vec<GrammarRule>> = LazyLock::new(
    || match serde_json::from_str::<GrammarStoreValue>(GRAMMAR_DATA) {
        Ok(content) => content.grammar,
        Err(e) => {
            error!("Failed to parse grammar.json: {}", e);
            panic!("Failed to parse grammar.json: {}", e);
        }
    },
);

pub fn get_rule_by_id(rule_id: &Ulid) -> Option<&'static GrammarRule> {
    GRAMMAR_RULES.iter().find(|x| x.rule_id() == rule_id)
}
