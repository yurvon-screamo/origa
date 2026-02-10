use std::sync::LazyLock;

use serde::Deserialize;
use ulid::Ulid;

use crate::domain::GrammarRule;

const GRAMMAR_DATA: &str = include_str!("./grammar.json");

#[derive(Deserialize)]
struct GrammarStoreValue {
    grammar: Vec<GrammarRule>,
}

pub static GRAMMAR_RULES: LazyLock<Vec<GrammarRule>> = LazyLock::new(|| {
    let content: GrammarStoreValue = serde_json::from_str(GRAMMAR_DATA).unwrap();
    content.grammar
});

pub fn get_rule_by_id(rule_id: &Ulid) -> Option<&'static GrammarRule> {
    GRAMMAR_RULES.iter().find(|x| x.rule_id() == rule_id)
}
