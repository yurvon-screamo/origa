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
    match serde_json::from_str::<GrammarStoreValue>(GRAMMAR_DATA) {
        Ok(content) => content.grammar,
        Err(e) => {
            // In Wasm, we might want to know why it failed
            #[cfg(target_arch = "wasm32")]
            web_sys::console::error_1(&format!("Failed to parse grammar.json: {}", e).into());

            panic!("Failed to parse grammar.json: {}", e);
        }
    }
});

pub fn get_rule_by_id(rule_id: &Ulid) -> Option<&'static GrammarRule> {
    GRAMMAR_RULES.iter().find(|x| x.rule_id() == rule_id)
}
