//! Per-token anomaly classification.
//!
//! ``classify_token`` inspects one ``TokenTranslation`` against the FAIL/WARN
//! taxonomy defined in ``super::ProblemKind`` and emits one report per
//! ``(token, problem_kind)`` pair so a single token can surface multiple
//! issues (e.g. W2 + W4 simultaneously).

use std::collections::HashSet;
use std::sync::OnceLock;

use origa::dictionary::grammar::iter_grammar_rules;
use origa::domain::{JapaneseChar, PartOfSpeech, TokenTranslation};

use super::harness::{ProblemKind, ProblemReport};

/// Union of every grammar keyword across all rules. Used by F3 to detect
/// particles/auxiliaries that *should* have a grammar_label but don't.
fn grammar_keywords_union() -> &'static HashSet<String> {
    static UNION: OnceLock<HashSet<String>> = OnceLock::new();
    UNION.get_or_init(|| {
        iter_grammar_rules()
            .flat_map(|rule| rule.keywords().iter().flatten().cloned())
            .collect()
    })
}

fn is_katakana_only(text: &str) -> bool {
    !text.is_empty() && text.chars().all(|c| c.is_katakana() || c == 'ー')
}

fn is_primary_vocab(pos: &PartOfSpeech) -> bool {
    matches!(
        pos,
        PartOfSpeech::Noun | PartOfSpeech::Verb | PartOfSpeech::IAdjective
    )
}

fn is_aux_like(pos: &PartOfSpeech) -> bool {
    matches!(pos, PartOfSpeech::Particle | PartOfSpeech::AuxiliaryVerb)
}

pub fn classify_token(
    translation: &TokenTranslation,
    phrase_idx: usize,
    phrase_text: &str,
    token_idx: usize,
) -> Vec<ProblemReport> {
    let mut reports = Vec::new();
    let pos = &translation.pos;
    let aux_like = is_aux_like(pos);
    let keywords = grammar_keywords_union();

    check_empty_base(
        translation,
        phrase_idx,
        phrase_text,
        token_idx,
        &mut reports,
    );
    check_missing_grammar_label_for_keyword(
        translation,
        aux_like,
        keywords,
        phrase_idx,
        phrase_text,
        token_idx,
        &mut reports,
    );
    check_suspicious_katakana_base(
        translation,
        pos,
        aux_like,
        phrase_idx,
        phrase_text,
        token_idx,
        &mut reports,
    );
    check_unknown_word_no_translation(
        translation,
        pos,
        phrase_idx,
        phrase_text,
        token_idx,
        &mut reports,
    );
    check_conjugated_no_grammar_label(
        translation,
        pos,
        phrase_idx,
        phrase_text,
        token_idx,
        &mut reports,
    );
    check_aux_no_grammar_label(
        translation,
        aux_like,
        keywords,
        phrase_idx,
        phrase_text,
        token_idx,
        &mut reports,
    );
    check_incomplete_dictionary_entry(
        translation,
        phrase_idx,
        phrase_text,
        token_idx,
        &mut reports,
    );

    reports
}

fn make_report(
    kind: ProblemKind,
    detail: String,
    translation: &TokenTranslation,
    phrase_idx: usize,
    phrase_text: &str,
    token_idx: usize,
) -> ProblemReport {
    ProblemReport {
        phrase_idx,
        phrase_text: phrase_text.to_string(),
        token_idx,
        surface: translation.surface_form.clone(),
        base: translation.base_form.clone(),
        pos: format!("{:?}", translation.pos),
        kind,
        detail,
    }
}

fn check_empty_base(
    translation: &TokenTranslation,
    phrase_idx: usize,
    phrase_text: &str,
    token_idx: usize,
    reports: &mut Vec<ProblemReport>,
) {
    if translation.base_form.is_empty() {
        reports.push(make_report(
            ProblemKind::EmptyBaseForm,
            "base_form is empty".into(),
            translation,
            phrase_idx,
            phrase_text,
            token_idx,
        ));
    }
}

fn check_missing_grammar_label_for_keyword(
    translation: &TokenTranslation,
    aux_like: bool,
    keywords: &HashSet<String>,
    phrase_idx: usize,
    phrase_text: &str,
    token_idx: usize,
    reports: &mut Vec<ProblemReport>,
) {
    if aux_like
        && keywords.contains(&translation.surface_form)
        && translation.grammar_label.is_none()
    {
        reports.push(make_report(
            ProblemKind::MissingGrammarLabelKnownAux,
            format!(
                "「{}」 matches a grammar keyword but grammar_label is None",
                translation.surface_form
            ),
            translation,
            phrase_idx,
            phrase_text,
            token_idx,
        ));
    }
}

fn check_suspicious_katakana_base(
    translation: &TokenTranslation,
    pos: &PartOfSpeech,
    aux_like: bool,
    phrase_idx: usize,
    phrase_text: &str,
    token_idx: usize,
    reports: &mut Vec<ProblemReport>,
) {
    let verb_or_iadj = matches!(pos, PartOfSpeech::Verb | PartOfSpeech::IAdjective);
    if !verb_or_iadj && !aux_like {
        return;
    }
    if !is_katakana_only(&translation.base_form) {
        return;
    }

    let label = if verb_or_iadj {
        format!("katakana-only base_form '{}'", translation.base_form)
    } else {
        format!(
            "aux-like katakana-only base_form '{}'",
            translation.base_form
        )
    };
    reports.push(make_report(
        ProblemKind::SuspiciousKatakanaBase,
        label,
        translation,
        phrase_idx,
        phrase_text,
        token_idx,
    ));
}

fn check_unknown_word_no_translation(
    translation: &TokenTranslation,
    pos: &PartOfSpeech,
    phrase_idx: usize,
    phrase_text: &str,
    token_idx: usize,
    reports: &mut Vec<ProblemReport>,
) {
    if is_primary_vocab(pos) && translation.translation.is_none() {
        reports.push(make_report(
            ProblemKind::UnknownWordNoTranslation,
            "primary vocab word has no translation".into(),
            translation,
            phrase_idx,
            phrase_text,
            token_idx,
        ));
    }
}

fn check_conjugated_no_grammar_label(
    translation: &TokenTranslation,
    pos: &PartOfSpeech,
    phrase_idx: usize,
    phrase_text: &str,
    token_idx: usize,
    reports: &mut Vec<ProblemReport>,
) {
    if is_primary_vocab(pos)
        && translation.surface_form != translation.base_form
        && translation.grammar_label.is_none()
    {
        reports.push(make_report(
            ProblemKind::ConjugatedNoGrammarLabel,
            format!(
                "conjugated ({} -> {}) without grammar_label",
                translation.base_form, translation.surface_form
            ),
            translation,
            phrase_idx,
            phrase_text,
            token_idx,
        ));
    }
}

fn check_aux_no_grammar_label(
    translation: &TokenTranslation,
    aux_like: bool,
    keywords: &HashSet<String>,
    phrase_idx: usize,
    phrase_text: &str,
    token_idx: usize,
    reports: &mut Vec<ProblemReport>,
) {
    if aux_like
        && !keywords.contains(&translation.surface_form)
        && translation.grammar_label.is_none()
    {
        reports.push(make_report(
            ProblemKind::AuxiliaryNoGrammarLabel,
            format!(
                "aux-like '{}' outside grammar keyword union",
                translation.surface_form
            ),
            translation,
            phrase_idx,
            phrase_text,
            token_idx,
        ));
    }
}

fn check_incomplete_dictionary_entry(
    translation: &TokenTranslation,
    phrase_idx: usize,
    phrase_text: &str,
    token_idx: usize,
    reports: &mut Vec<ProblemReport>,
) {
    if translation
        .translation
        .as_ref()
        .is_some_and(|t| t.trim().is_empty())
    {
        reports.push(make_report(
            ProblemKind::IncompleteDictionaryEntry,
            "translation is Some but empty/whitespace".into(),
            translation,
            phrase_idx,
            phrase_text,
            token_idx,
        ));
    }
}
