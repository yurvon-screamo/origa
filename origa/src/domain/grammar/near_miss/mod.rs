//! Near-miss distractor generation for grammar quizzes (#503).
//!
//! A quiz option set is solvable by pattern spotting when the rule's own
//! visible tail (postfix or conjugation ending) appears in exactly one
//! option. Near-miss distractors keep the correct answer's ending intact
//! and mutate the stem instead, modeling the mistakes a learner actually
//! makes: forgot-to-conjugate, wrong vowel row, small-っ confusion, and
//! voicing confusion (た⇄だ).
//!
//! This module is pure string/kana logic over the row tables in
//! [`crate::domain::grammar::forms_verb`]; candidates that coincide with
//! a valid form of the same word are filtered out here (the "form soup"
//! contract), so every returned candidate is a broken form by
//! construction.

mod operators;
#[cfg(test)]
mod tests;

use crate::dictionary::grammar::FormatAction;
use crate::domain::PartOfSpeech;
use crate::domain::grammar::forms_verb::VerbGroup;

/// A correct form decomposed into the parts the operators mutate:
/// `prefix + stem + ending`. `prefix` is non-empty only for constructions
/// that prepend material the source word does not contain (the honorific
/// お-wrap: お書きになります = お + 書き + になります).
#[derive(Debug, Clone, PartialEq, Eq)]
struct Decomposed {
    prefix: String,
    stem: String,
    ending: String,
}

impl Decomposed {
    fn new(stem: &str, ending: &str) -> Self {
        Self {
            prefix: String::new(),
            stem: stem.to_string(),
            ending: ending.to_string(),
        }
    }
}

/// Ordered near-miss candidates for the correct form of `word` under a
/// pure conjugation chain (no postfix tail). Ordered by learner-error
/// plausibility, deduplicated, and guaranteed != `correct`.
///
/// Candidates that coincide with a *valid* form of the same word produced
/// by another conjugation action (書ける masquerading as a broken 書きます)
/// are filtered out while the ending is non-empty — that is the "form
/// soup" the quiz must not regress into. Stem-only forms (empty ending,
/// e.g. the 命令形 quiz 書け) are exempt: their whole option space is
/// stems, so a wrong-row stem is precisely the learner error under test.
pub(crate) fn near_miss_candidates(word: &str, correct: &str, pos: &PartOfSpeech) -> Vec<String> {
    let Some(parts) = decompose(word, correct, pos) else {
        return Vec::new();
    };

    let candidates: Vec<String> = match pos {
        PartOfSpeech::Verb => operators::verb_candidates(word, &parts),
        PartOfSpeech::IAdjective | PartOfSpeech::NaAdjective => {
            operators::adjective_candidates(word, &parts)
        },
        _ => Vec::new(),
    };

    let mut unique: Vec<String> = Vec::with_capacity(candidates.len());
    let mut valid_forms: Option<std::collections::HashSet<String>> = None;
    for candidate in candidates {
        if candidate == correct || candidate.is_empty() || unique.contains(&candidate) {
            continue;
        }
        if !parts.ending.is_empty() {
            // Lazily cache the valid forms of the word once (≤31 actions),
            // then filter every remaining candidate against the set.
            let valid = valid_forms.get_or_insert_with(|| valid_forms_of_word(word, pos));
            if valid.contains(&candidate) {
                continue;
            }
        }
        unique.push(candidate);
    }
    unique
}

/// All valid conjugations of `word` via the actions of its
/// part-of-speech group — the "form soup" the quiz must not offer.
fn valid_forms_of_word(word: &str, pos: &PartOfSpeech) -> std::collections::HashSet<String> {
    let actions: &[FormatAction] = match pos {
        PartOfSpeech::Verb => FormatAction::all_verb_actions(),
        PartOfSpeech::IAdjective => FormatAction::all_i_adjective_actions(),
        PartOfSpeech::NaAdjective => FormatAction::all_na_adjective_actions(),
        _ => return std::collections::HashSet::new(),
    };
    actions
        .iter()
        .filter_map(|action| {
            crate::domain::apply_format_actions(word, std::slice::from_ref(action), pos).ok()
        })
        .collect()
}

fn decompose(word: &str, correct: &str, pos: &PartOfSpeech) -> Option<Decomposed> {
    match pos {
        PartOfSpeech::Verb => decompose_verb(word, correct),
        PartOfSpeech::IAdjective | PartOfSpeech::NaAdjective => decompose_adjective(word, correct),
        _ => None,
    }
}

/// Longest common character prefix of two strings.
fn common_prefix_len(a: &str, b: &str) -> usize {
    a.chars().zip(b.chars()).take_while(|(x, y)| x == y).count()
}

fn decompose_verb(word: &str, correct: &str) -> Option<Decomposed> {
    // Honorific wrap: the correct form starts with material the word does
    // not contain (お書きになります for 書く). Recurse on the wrapped part and
    // re-attach the prefix to every candidate.
    let prefix_len = common_prefix_len(word, correct);
    if prefix_len == 0 && correct.starts_with('お') && word.chars().count() > 1 {
        let inner = decompose_verb(word, &correct['お'.len_utf8()..])?;
        let wrapped = Decomposed {
            prefix: "お".to_string(),
            stem: inner.stem,
            ending: inner.ending,
        };
        return Some(wrapped);
    }

    let word_chars: Vec<char> = word.chars().collect();
    let correct_chars: Vec<char> = correct.chars().collect();
    if word_chars.is_empty() || correct_chars.len() <= prefix_len {
        return None;
    }

    let group = crate::domain::grammar::forms_verb::classify_verb(word);
    // する-compounds conjugate on the す row (勉強する → 勉強し + ます) and
    // 来る/くる alternate to hiragana stems (来る → きます/こない: the kanji
    // shares no prefix with き/こ, so the godan formula's empty-prefix
    // branch handles them), so all three decompose like godan despite
    // being classified irregular.
    let godan_like = matches!(group, VerbGroup::Godan)
        || word == "する"
        || word == "くる"
        || word == "来る"
        || word.ends_with("する");

    if godan_like {
        // word = P + X, correct = P + Y + ending: skip the conjugated kana.
        if correct_chars.len() < prefix_len + 1 {
            return None;
        }
        let stem: String = correct_chars[..=prefix_len].iter().collect();
        let ending: String = correct_chars[prefix_len + 1..].iter().collect();
        Some(Decomposed::new(&stem, &ending))
    } else {
        // Ichidan: word = P + る, correct = P + ending.
        let stem: String = correct_chars[..prefix_len].iter().collect();
        let ending: String = correct_chars[prefix_len..].iter().collect();
        if stem.is_empty() {
            return None;
        }
        Some(Decomposed::new(&stem, &ending))
    }
}

fn decompose_adjective(word: &str, correct: &str) -> Option<Decomposed> {
    let prefix_len = common_prefix_len(word, correct);
    let correct_chars: Vec<char> = correct.chars().collect();
    if prefix_len == 0 || correct_chars.len() <= prefix_len {
        return None;
    }
    // 高い → 高かった: the い is replaced by a longer ending, so — unlike
    // verbs — the conjugated kana belongs to the ending, not the stem.
    // Na-adjectives keep the whole word as the stem (静か + で), which the
    // fully-consumed prefix expresses naturally.
    let stem: String = correct_chars[..prefix_len].iter().collect();
    let ending: String = correct_chars[prefix_len..].iter().collect();
    Some(Decomposed::new(&stem, &ending))
}
