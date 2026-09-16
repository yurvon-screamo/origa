//! Grammar quiz generation (#503).
//!
//! A quiz option set is solvable by pattern spotting when the rule's own
//! visible tail (postfix or conjugation ending) appears in exactly one
//! option. Every distractor therefore keeps the correct answer's tail and
//! mutates the stem instead — the mistakes a learner actually makes:
//!
//! - postfix-tail chains (`～つもりです`, `～てください`): another full form
//!   of the word carries the same postfix (書いてつもりです,
//!   書いたつもりです — often a valid construction of a *different* rule,
//!   which tests the meaning distinction the rule body teaches);
//! - pure conjugation chains (`～ます`, `～て`): kana-level near-misses via
//!   [`near_miss_candidates`] (書かます, 書くない, 書って — broken stems,
//!   never valid forms of the same word).
//!
//! Deterministic priority order; fewer than `count` results means the
//! caller must skip the question (no fallback pools).

mod tail;
#[cfg(test)]
mod tests;

use std::collections::HashSet;

use rand::Rng;
use rand::prelude::SliceRandom;

use crate::dictionary::grammar::{FormatAction, FormatActionGroup, GrammarRule};
use crate::domain::knowledge::KnowledgeSet;
use crate::domain::{Card, OrigaError, PartOfSpeech};

use super::near_miss::near_miss_candidates;
use tail::composed_tail_candidates;

#[derive(Debug, Clone, PartialEq)]
pub struct GrammarPracticeQuestion {
    word_text: String,
    options: Vec<String>,
    correct_index: usize,
}

impl GrammarPracticeQuestion {
    pub fn word_text(&self) -> &str {
        &self.word_text
    }

    pub fn options(&self) -> &[String] {
        &self.options
    }

    pub fn correct_index(&self) -> usize {
        self.correct_index
    }
}

pub fn generate_grammar_practice_questions(
    rule: &GrammarRule,
    knowledge_set: &KnowledgeSet,
    count: usize,
    rng: &mut impl Rng,
) -> Result<Vec<GrammarPracticeQuestion>, OrigaError> {
    let pos = rule
        .apply_to()
        .into_iter()
        .next()
        .ok_or_else(|| OrigaError::GrammarFormatError {
            reason: "Rule has no supported parts of speech".to_string(),
        })?;

    let mut words = find_known_vocab_words_for_pos(knowledge_set, &pos);
    words.shuffle(rng);

    let selected_words: Vec<String> = words.into_iter().take(count).collect();

    let mut questions = Vec::with_capacity(selected_words.len());

    for word in selected_words {
        let correct = rule.format(&word, &pos)?;

        let actions = match rule.format_actions_for_pos(&pos) {
            Some(a) => a,
            None => continue,
        };

        let distractors = generate_grammar_distractors(actions, &word, &pos, &correct, 3);

        if distractors.len() < 3 {
            continue;
        }

        let mut options = distractors;
        let correct_index = rng.random_range(0..=options.len());
        options.insert(correct_index, correct);

        questions.push(GrammarPracticeQuestion {
            word_text: word,
            options,
            correct_index,
        });
    }

    Ok(questions)
}

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

/// Near-miss distractors for a grammar quiz; see the module docs.
pub fn generate_grammar_distractors(
    rules: &[FormatAction],
    source_word: &str,
    pos: &PartOfSpeech,
    correct_text: &str,
    count: usize,
) -> Vec<String> {
    let mut distractors = Vec::with_capacity(count);
    let mut seen = HashSet::new();
    seen.insert(correct_text.to_string());

    for candidate in distractor_candidates(rules, source_word, pos, correct_text) {
        if distractors.len() >= count {
            break;
        }
        if !candidate.is_empty() && seen.insert(candidate.clone()) {
            distractors.push(candidate);
        }
    }
    distractors
}

fn distractor_candidates(
    rules: &[FormatAction],
    source_word: &str,
    pos: &PartOfSpeech,
    correct_text: &str,
) -> Vec<String> {
    // Tail = maximal suffix of Universal actions; base = conjugating prefix.
    let tail_start = rules
        .iter()
        .rposition(|a| a.group() != FormatActionGroup::Universal)
        .map_or(0, |i| i + 1);
    let (base, tail) = rules.split_at(tail_start);
    let postfix_tail = !tail.is_empty()
        && tail
            .iter()
            .all(|a| matches!(a, FormatAction::AddPostfix { .. }));

    if postfix_tail {
        composed_tail_candidates(base, tail, source_word, pos)
    } else {
        near_miss_candidates(source_word, correct_text, pos)
    }
}
