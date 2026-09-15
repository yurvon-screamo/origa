//! Near-miss mutation operators (#503).
//!
//! Every operator takes the decomposed correct form (`prefix + stem +
//! ending`) plus the source word and rebuilds a *broken* variant that
//! keeps the ending visible: the ending is what identifies the rule, the
//! stem is what the learner must produce correctly.

use super::Decomposed;
use crate::domain::grammar::forms_verb::{
    GODAN_TO_IMPERATIVE, GODAN_TO_MIZENKEI, GODAN_TO_O_ROW, GODAN_TO_STEM,
};

/// Bidirectional dakuten toggle. Voiced ⇄ voiceless pairs only; kana
/// without a pair (already-voiced ん-row, ま, ら, …) make [`voice_swap`]
/// yield no candidate.
const DAKUTEN_SWAP: &[(char, char)] = &[
    ('か', 'が'),
    ('き', 'ぎ'),
    ('く', 'ぐ'),
    ('け', 'げ'),
    ('こ', 'ご'),
    ('さ', 'ざ'),
    ('し', 'じ'),
    ('す', 'ず'),
    ('せ', 'ぜ'),
    ('そ', 'ぞ'),
    ('た', 'だ'),
    ('ち', 'ぢ'),
    ('つ', 'づ'),
    ('て', 'で'),
    ('と', 'ど'),
    ('は', 'ば'),
    ('ひ', 'び'),
    ('ふ', 'ぶ'),
    ('へ', 'べ'),
    ('ほ', 'ぼ'),
];

fn toggle_dakuten(kana: char) -> Option<char> {
    DAKUTEN_SWAP
        .iter()
        .find(|(plain, voiced)| *plain == kana || *voiced == kana)
        .map(|&(plain, voiced)| if kana == plain { voiced } else { plain })
}

fn ending_first(parts: &Decomposed) -> Option<char> {
    parts.ending.chars().next()
}

/// forgot-to-conjugate: the raw word glued to the ending (書くない,
/// 食べるます, 高いく).
fn forgot(word: &str, parts: &Decomposed) -> Option<String> {
    Some(format!("{}{}{}", parts.prefix, word, parts.ending))
}

/// Wrong godan row: the word's conjugation kana swapped to another vowel
/// row before the ending (書かます, 書けます). `table` picks the row.
/// する-compounds conjugate on the す (勉強する → 勉強さます), so the
/// swapped kana is the one before the final る.
fn wrong_row(word: &str, parts: &Decomposed, table: &[(char, &str)]) -> Option<String> {
    let mut word_chars: Vec<char> = word.chars().collect();
    let is_suru_compound = word == "する" || word.ends_with("する");
    let last = if is_suru_compound {
        word_chars.pop()?;
        word_chars.pop()?
    } else {
        word_chars.pop()?
    };
    let replacement = table
        .iter()
        .find(|(kana, _)| *kana == last)
        .map(|(_, row)| *row)?;
    Some(format!(
        "{}{}{}{}",
        parts.prefix,
        word_chars.iter().collect::<String>(),
        replacement,
        parts.ending
    ))
}

/// Small-っ confusion, verb flavor: the stem's conjugated kana is replaced
/// by っ (書い + て → 書って). Skipped when nothing of the stem would
/// remain (来 + ます would otherwise degrade to bare っます).
fn verb_tsu(parts: &Decomposed) -> Option<String> {
    let stem_len = parts.stem.chars().count();
    if stem_len == 0 {
        return None;
    }
    let stripped: String = parts.stem.chars().take(stem_len - 1).collect();
    if stripped.is_empty() {
        return None;
    }
    Some(format!("{}{}っ{}", parts.prefix, stripped, parts.ending))
}

/// Small-っ confusion, adjective/ichidan flavor: っ inserted between stem
/// and ending (高 + っ + く → 高っく, 食べ + っ + ます → 食べっます).
fn insert_tsu(parts: &Decomposed) -> Option<String> {
    Some(format!("{}{}っ{}", parts.prefix, parts.stem, parts.ending))
}

/// Voicing confusion: the ending's first kana toggles dakuten (書い + て →
/// 書いで, 高 + く → 高ぐ, and back: 高 + がる → 高かる).
fn voice_swap(parts: &Decomposed) -> Option<String> {
    let first = ending_first(parts)?;
    let swapped = toggle_dakuten(first)?;
    let rest: String = parts.ending.chars().skip(1).collect();
    Some(format!("{}{}{}{}", parts.prefix, parts.stem, swapped, rest))
}

/// っ-drop: a っ at the start of the ending or right after its first kana
/// is a classic typo to drop (高か + った → 高かた, 取っ + て → 取て).
fn tsu_drop(parts: &Decomposed) -> Option<String> {
    let chars: Vec<char> = parts.ending.chars().collect();
    let drop_at = match chars.as_slice() {
        ['っ', ..] => Some(0),
        [_, 'っ', ..] => Some(1),
        _ => None,
    }?;
    let kept: String = chars
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != drop_at)
        .map(|(_, c)| *c)
        .collect();
    Some(format!("{}{}{}", parts.prefix, parts.stem, kept))
}

/// Verb candidates in learner-error plausibility order: forgot → a-row →
/// っ → voicing → e-row → o-row → i-row → っ-drop. The row swaps also
/// cover ichidan verbs: swapping the dropped る rebuilds the classic
/// godan-ization mistake (食べ + り + ます → 食べります).
pub(super) fn verb_candidates(word: &str, parts: &Decomposed) -> Vec<String> {
    vec![
        forgot(word, parts),
        wrong_row(word, parts, GODAN_TO_MIZENKEI),
        verb_tsu(parts),
        voice_swap(parts),
        wrong_row(word, parts, GODAN_TO_IMPERATIVE),
        wrong_row(word, parts, GODAN_TO_O_ROW),
        wrong_row(word, parts, GODAN_TO_STEM),
        tsu_drop(parts),
    ]
    .into_iter()
    .flatten()
    .collect()
}

/// Adjective candidates: forgot-glued → っ-insert → voicing → っ-drop.
/// Na-adjectives simply produce fewer applicable candidates (their kanji
/// stem has no kana to mutate), which callers translate into "no quiz".
pub(super) fn adjective_candidates(word: &str, parts: &Decomposed) -> Vec<String> {
    vec![
        forgot(word, parts),
        insert_tsu(parts),
        voice_swap(parts),
        tsu_drop(parts),
    ]
    .into_iter()
    .flatten()
    .collect()
}
