use origa::dictionary::phrase::{get_phrases_by_token, is_phrases_loaded};
use origa::domain::{Card, StudyCard};
use origa::use_cases::collect_known_grammar_rules;
use ulid::Ulid;

use std::collections::HashMap;

pub fn find_ready_phrases(word: &str, study_cards: &HashMap<Ulid, StudyCard>) -> Vec<Ulid> {
    if !is_phrases_loaded() {
        return Vec::new();
    }

    let candidates = get_phrases_by_token(word);
    if candidates.is_empty() {
        return Vec::new();
    }

    let known_grammar = collect_known_grammar_rules(study_cards);

    candidates
        .iter()
        .filter(|entry| {
            entry
                .tokens()
                .iter()
                .all(|token| is_known_vocabulary_word(token, study_cards))
                && entry
                    .grammar_rules()
                    .iter()
                    .all(|rule_id| known_grammar.contains(rule_id))
        })
        .map(|entry| *entry.id())
        .collect()
}

fn is_known_vocabulary_word(word: &str, study_cards: &HashMap<Ulid, StudyCard>) -> bool {
    study_cards.values().any(|sc| {
        matches!(sc.card(), Card::Vocabulary(vocab)
            if vocab.word().text() == word
            && sc.memory().is_known_card())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_ready_phrases_returns_empty_when_db_not_loaded() {
        let study_cards = HashMap::new();
        let result = find_ready_phrases("test", &study_cards);
        assert!(result.is_empty());
    }

    #[test]
    fn is_known_vocabulary_word_returns_false_for_empty_study_cards() {
        let study_cards = HashMap::new();
        assert!(!is_known_vocabulary_word("猫", &study_cards));
    }

    #[test]
    fn collect_known_grammar_rules_returns_empty_for_empty_study_cards() {
        let study_cards = HashMap::new();
        let rules = collect_known_grammar_rules(&study_cards);
        assert!(rules.is_empty());
    }
}
