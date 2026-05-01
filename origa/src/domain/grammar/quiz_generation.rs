use std::collections::HashSet;

use rand::Rng;
use rand::prelude::IndexedRandom;

use crate::dictionary::grammar::{FormatAction, FormatActionGroup};
use crate::domain::knowledge::KnowledgeSet;
use crate::domain::{Card, PartOfSpeech, apply_format_actions};

pub fn find_known_vocab_words_for_pos(
    knowledge_set: &KnowledgeSet,
    pos: &PartOfSpeech,
) -> Vec<String> {
    knowledge_set
        .study_cards()
        .values()
        .filter(|sc| sc.memory().is_known_card() || sc.memory().is_in_progress())
        .filter_map(|sc| match sc.card() {
            Card::Vocabulary(v) => {
                let word = v.word().text().to_string();
                let vocab_pos = v.part_of_speech().ok()?;
                if vocab_pos == *pos { Some(word) } else { None }
            },
            _ => None,
        })
        .collect()
}

pub fn generate_grammar_distractors(
    rules: &[FormatAction],
    source_word: &str,
    pos: &PartOfSpeech,
    correct_text: &str,
    count: usize,
    rng: &mut impl Rng,
) -> Vec<String> {
    let mut distractors = Vec::new();
    let mut seen = HashSet::new();
    seen.insert(correct_text.to_string());

    let max_attempts = count * 10;
    let mut attempts = 0;

    while distractors.len() < count && attempts < max_attempts {
        attempts += 1;

        if let Some(distractor) = apply_mutated_pattern(rules, source_word, pos, rng) {
            if !seen.contains(&distractor) {
                seen.insert(distractor.clone());
                distractors.push(distractor);
            }
        }
    }

    distractors
}

pub fn apply_mutated_pattern(
    rules: &[FormatAction],
    source_word: &str,
    pos: &PartOfSpeech,
    rng: &mut impl Rng,
) -> Option<String> {
    let mutable_indices: Vec<usize> = rules
        .iter()
        .enumerate()
        .filter(|(_, a)| a.group() != FormatActionGroup::Universal)
        .map(|(i, _)| i)
        .collect();

    if mutable_indices.is_empty() {
        return None;
    }

    let idx = mutable_indices.choose(rng)?;
    let original_action = &rules[*idx];
    let alternatives = original_action.mutation_alternatives();

    if alternatives.is_empty() {
        return None;
    }

    let alternative = alternatives.choose(rng)?;
    let mut mutated: Vec<FormatAction> = rules.to_vec();
    mutated[*idx] = (*alternative).clone();

    apply_format_actions(source_word, &mutated, pos).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_mutated_pattern_returns_none_for_universal_only() {
        let actions = vec![FormatAction::AddPostfix {
            postfix: "test".into(),
        }];
        let result = apply_mutated_pattern(&actions, "word", &PartOfSpeech::Verb, &mut rand::rng());
        assert!(result.is_none());
    }

    #[test]
    fn generate_distractors_excludes_correct() {
        let actions = vec![FormatAction::VerbToMasu {}];
        let distractors = generate_grammar_distractors(
            &actions,
            "行く",
            &PartOfSpeech::Verb,
            "行きます",
            3,
            &mut rand::rng(),
        );
        assert!(distractors.iter().all(|d| d != "行きます"));
    }

    #[test]
    fn generate_distractors_no_duplicates() {
        let actions = vec![FormatAction::VerbToMasu {}];
        let distractors = generate_grammar_distractors(
            &actions,
            "行く",
            &PartOfSpeech::Verb,
            "行きます",
            3,
            &mut rand::rng(),
        );
        let unique: HashSet<_> = distractors.iter().collect();
        assert_eq!(unique.len(), distractors.len());
    }

    #[test]
    fn generate_distractors_returns_up_to_count() {
        let actions = vec![FormatAction::VerbToMasu {}];
        let distractors = generate_grammar_distractors(
            &actions,
            "行く",
            &PartOfSpeech::Verb,
            "行きます",
            3,
            &mut rand::rng(),
        );
        assert!(distractors.len() <= 3);
    }

    #[test]
    fn find_known_vocab_returns_empty_for_empty_set() {
        let ks = KnowledgeSet::new();
        let result = find_known_vocab_words_for_pos(&ks, &PartOfSpeech::Verb);
        assert!(result.is_empty());
    }
}
