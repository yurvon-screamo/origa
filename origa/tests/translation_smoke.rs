//! Smoke test for the translation pipeline over the p0000.json phrase corpus.
//!
//! Runs every phrase in `cdn/phrases/data/p0000.json` (~771 entries) through
//! `tokenize_text` + `lookup_tokens_translations` and classifies anomalies
//! using the FAIL/WARN taxonomy defined in `harness::ProblemKind`.
//!
//! - Slice-1 entry point: `should_load_phrase_chunk_and_tokenize_first_phrase`.
//! - Slice-2 discovery harness: `discover_translation_problems_russian` and
//!   `discover_translation_problems_english` (`#[ignore]`, run with
//!   `--nocapture --ignored`).
//! - Slice-3 regression tests: parameterized cases that pin down fixes.

#[path = "translation_smoke/bootstrap.rs"]
mod bootstrap;
#[path = "translation_smoke/classify.rs"]
mod classify;
#[path = "translation_smoke/data.rs"]
mod data;
#[path = "translation_smoke/harness.rs"]
mod harness;
#[path = "translation_smoke/report.rs"]
mod report;

use rstest::rstest;

use origa::domain::{JapaneseChar, NativeLanguage, lookup_tokens_translations, tokenize_text};

use bootstrap::ensure_all_dictionaries;
use data::load_phrase_texts_from_chunk;

#[test]
fn should_load_phrase_chunk_and_tokenize_first_phrase() {
    if !ensure_all_dictionaries() {
        eprintln!(
            "skip: cdn/ artifacts absent \
             (cdn/ is gitignored; restore via scripts/deploy_cdn.py)"
        );
        return;
    }

    let phrases = match load_phrase_texts_from_chunk("p0000.json") {
        Some(texts) => texts,
        None => {
            eprintln!(
                "skip: cdn/phrases/data/p0000.json absent \
                 (cdn/ is gitignored; restore via scripts/deploy_cdn.py)"
            );
            return;
        },
    };

    assert!(
        !phrases.is_empty(),
        "p0000.json must contain at least one phrase"
    );

    let first = &phrases[0];
    let tokens = tokenize_text(first).expect("first phrase must tokenize");
    let translations = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, first);

    assert!(
        !translations.is_empty(),
        "first phrase must produce at least one token translation"
    );
}

/// Slice-3 RED tests: when a phrase is written in hiragana but Lindera resolves
/// the lemma in kanji (e.g. ``わかる`` → base ``分かる``), the grammar_label
/// resolver must still match conjugated forms. ``find_format_map_matches``
/// formats the base into its conjugated surface (``分かって``) and checks the
/// original text — but the kanji-formatted string never appears in a hiragana
/// phrase, so the match silently fails.
///
/// Discovered by the smoke harness (W2 category) across 1k+ phrases. Each case
/// below is a minimal hiragana-only reproduction (text contains no kanji) so
/// the new hiragana-base fallback path is the only way the resolver can
/// succeed.
#[rstest]
#[case("わかってる", "分かる")]
#[case("もらっている", "貰う")]
#[case("わかってるなら", "分かる")]
fn should_resolve_grammar_label_for_hiragana_phrase_with_kanji_base(
    #[case] phrase: &str,
    #[case] expected_base: &str,
) {
    // Arrange — guard against accidental kanji in the fixture itself
    assert!(
        !phrase.chars().any(|c| c.is_kanji()),
        "test fixture '{phrase}' must be pure hiragana, \
         otherwise the new fallback path is not exercised"
    );
    if !ensure_all_dictionaries() {
        eprintln!(
            "skip: cdn/ artifacts absent \
             (cdn/ is gitignored; restore via scripts/deploy_cdn.py)"
        );
        return;
    }

    // Act
    let tokens = tokenize_text(phrase).expect("phrase must tokenize");
    let translations = lookup_tokens_translations(&tokens, &NativeLanguage::English, phrase);

    // Assert
    let target = translations
        .iter()
        .find(|t| t.base_form == expected_base)
        .unwrap_or_else(|| {
            panic!(
                "expected base_form 「{expected_base}」 in tokenization of 「{phrase}」, \
                 got: {translations:?}"
            )
        });

    assert!(
        target.grammar_label.is_some(),
        "「{expected_base}」 conjugated inside the hiragana phrase 「{phrase}」 \
         must carry a grammar_label, got: {target:?}"
    );
}
