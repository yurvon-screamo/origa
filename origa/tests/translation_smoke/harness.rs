//! Discovery harness for translation pipeline anomalies.
//!
//! Run with:
//!   ``cargo test -p origa --test translation_smoke -- --nocapture --ignored``
//!
//! The harness classifies every token of every phrase in ``p0000.json`` into
//! the FAIL/WARN categories formalised by the architect:
//!
//! - FAIL (TDD targets, deterministic detection):
//!   - **F1** ``TokenizeError`` — ``tokenize_text`` returned ``Err``.
//!   - **F3** ``MissingGrammarLabelKnownAux`` — ``Particle``/``AuxiliaryVerb``
//!     whose surface matches a grammar keyword but ``grammar_label`` is unset.
//!   - **F4** ``EmptyBaseForm`` — ``base_form`` is empty.
//!   - **F5** ``SuspiciousKatakanaBase`` — katakana-only ``base_form`` for a
//!     Verb/IAdj/Particle/AuxiliaryVerb (Lindera mixed-script limitation).
//!
//! - WARN (manual triage):
//!   - **W1** ``UnknownWordNoTranslation`` — Noun/Verb/IAdj with no translation.
//!   - **W2** ``ConjugatedNoGrammarLabel`` — Noun/Verb/IAdj with ``surface !=
//!     base`` and no grammar_label (format_map gap or kanji↔kana alternation).
//!   - **W3** ``AuxiliaryNoGrammarLabel`` — Particle/AuxiliaryVerb outside the
//!     grammar keyword union with no grammar_label.
//!   - **W4** ``IncompleteDictionaryEntry`` — ``translation`` is ``Some`` but
//!     empty/whitespace.
//!
//! F2 ("missing translation for a known word") is intentionally absent: the
//! architect proved it is structurally impossible — ``translation`` and
//! ``get_translation`` read the same vocabulary_map.
//!
//! Known findings on ``p0000.json`` (771 phrases):
//! - **F1 / F3 / F4** stayed at 0 from the first run — pipeline invariants hold.
//! - **F5** shows 3 hits, all ``AuxiliaryVerb`` rendered in katakana (e.g.
//!   ``デス`` for ``です`` in stylised speech, ``ム`` mis-tagged as aux by
//!   Lindera). These are upstream-Lindera / input-data quirks, not bugs in
//!   ``translation.rs``; the cases are logged for visibility but not TDD-fixed.
//! - **W2** dropped from 1367 → 901 after the
//!   ``should_resolve_grammar_label_for_hiragana_phrase_with_kanji_base``
//!   fix in ``translation.rs``. The residual is dominated by kanji↔kana
//!   alternations (``事 → こと``, ``為る → する``) and legitimate grammar-rule
//!   gaps; the harness reports them but cannot distinguish the two without
//!   ``phonological_base_form`` exposed on ``TokenTranslation``.
//! - **W3** (~4k) is by design: ``grammar.json`` keyword rules describe
//!   multi-token patterns (``～は～です``), not bare particles, so ``は / が / を``
//!   etc. never receive a grammar_label.
//! - **W4** stayed at 0 — every vocabulary entry that resolves to ``Some`` has
//!   at least one non-whitespace line.

use origa::domain::{NativeLanguage, tokenize_text};

use super::bootstrap::ensure_all_dictionaries;
use super::classify::classify_token;
use super::data::load_phrase_texts_from_chunk;
use super::report::print_report;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProblemKind {
    TokenizeError,
    MissingGrammarLabelKnownAux,
    EmptyBaseForm,
    SuspiciousKatakanaBase,
    UnknownWordNoTranslation,
    ConjugatedNoGrammarLabel,
    AuxiliaryNoGrammarLabel,
    IncompleteDictionaryEntry,
}

impl ProblemKind {
    pub fn as_code(self) -> &'static str {
        match self {
            ProblemKind::TokenizeError => "F1",
            ProblemKind::MissingGrammarLabelKnownAux => "F3",
            ProblemKind::EmptyBaseForm => "F4",
            ProblemKind::SuspiciousKatakanaBase => "F5",
            ProblemKind::UnknownWordNoTranslation => "W1",
            ProblemKind::ConjugatedNoGrammarLabel => "W2",
            ProblemKind::AuxiliaryNoGrammarLabel => "W3",
            ProblemKind::IncompleteDictionaryEntry => "W4",
        }
    }

    pub fn is_fail(self) -> bool {
        matches!(
            self,
            ProblemKind::TokenizeError
                | ProblemKind::MissingGrammarLabelKnownAux
                | ProblemKind::EmptyBaseForm
                | ProblemKind::SuspiciousKatakanaBase
        )
    }
}

#[derive(Debug, Clone)]
pub struct ProblemReport {
    pub phrase_idx: usize,
    pub phrase_text: String,
    pub token_idx: usize,
    pub surface: String,
    pub base: String,
    pub pos: String,
    pub kind: ProblemKind,
    pub detail: String,
}

/// Runs the full phrase corpus through the pipeline and collects every
/// anomaly. F1 errors short-circuit per-phrase (no token classification).
fn analyze_corpus(lang: NativeLanguage) -> Vec<ProblemReport> {
    let loaded = ensure_all_dictionaries();
    let phrases = match load_phrase_texts_from_chunk("p0000.json") {
        Some(texts) if loaded => texts,
        _ => {
            eprintln!(
                "skip discovery: cdn artifacts absent \
                 (cdn/ is gitignored; restore via scripts/deploy_cdn.py)"
            );
            return Vec::new();
        },
    };

    let mut reports = Vec::new();
    for (idx, text) in phrases.iter().enumerate() {
        match tokenize_text(text) {
            Ok(tokens) => {
                let translations = origa::domain::lookup_tokens_translations(&tokens, &lang, text);
                for (token_idx, translation) in translations.iter().enumerate() {
                    reports.extend(classify_token(translation, idx, text, token_idx));
                }
            },
            Err(err) => reports.push(ProblemReport {
                phrase_idx: idx,
                phrase_text: text.clone(),
                token_idx: 0,
                surface: String::new(),
                base: String::new(),
                pos: String::new(),
                kind: ProblemKind::TokenizeError,
                detail: format!("tokenize_text failed: {err:?}"),
            }),
        }
    }
    reports
}

#[test]
#[ignore = "discovery harness: dumps TSV + summary for Russian, run explicitly"]
pub fn discover_translation_problems_russian() {
    let reports = analyze_corpus(NativeLanguage::Russian);
    print_report(&reports);
}

#[test]
#[ignore = "discovery harness: dumps TSV + summary for English, run explicitly"]
pub fn discover_translation_problems_english() {
    let reports = analyze_corpus(NativeLanguage::English);
    print_report(&reports);
}
