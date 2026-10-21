//! Standalone masu-stem (ren'yōkei) resolution.
//!
//! A verb stem not followed by a conjugation suffix is a continuative or a
//! noun-ified stem; this resolver labels it 連用形 and steers the translation
//! fallback toward the noun entry. Split out of the former translation.rs
//! god-file (issue #796).

use crate::dictionary::grammar::GRAMMAR_RULES;
use crate::domain::NativeLanguage;
use crate::domain::tokenizer::{PartOfSpeech, TokenInfo};

use super::grammar_match::GrammarMatch;

/// Surfaces that, when following a verb masu-stem, mark it as the base of a
/// conjugated form (polite/te/past/conditional/...) rather than a standalone
/// continuative or noun. Empirically validated by the corpus audit POS gate:
/// た is always AuxiliaryVerb, て is mostly Particle, ば is mostly Particle.
/// Checking the surface avoids relying on POS alone, which would over-match
/// for て/ば (Particle) cases.
const CONJUGATION_SUFFIX_SURFACES: &[&str] = &[
    "ます",
    "ません",
    "まし",
    "ませ",
    "た",
    "て",
    "ば",
    "たら",
    "よう",
    "う",
    "たい",
    "ない",
];

fn next_token_is_conjugation_suffix(tokens: &[TokenInfo], index: usize) -> bool {
    let Some(next) = tokens.get(index + 1) else {
        return false;
    };
    if matches!(next.part_of_speech(), PartOfSpeech::AuxiliaryVerb) {
        return true;
    }
    let next_surface = next.orthographic_surface_form();
    CONJUGATION_SUFFIX_SURFACES.contains(&next_surface)
}

/// Standalone masu-stem (ren'yōkei) resolver. A verb stem that is NOT followed
/// by a conjugation suffix is either a continuative joining clauses or a
/// noun-ified stem (願い=request, 暮らし=living, 話=story). The resolver returns
/// the 連用形 grammar rule by title — it does NOT go through `find_format_map`
/// because a `{Verb: [VerbToStem]}` format_map would over-match Path B
/// (`detect_format_map_rules` → `enrich_phrases_with_grammar`) on every verb
/// in every text. Mirrors the `resolve_sou_da_match` pattern.
pub(super) fn resolve_masu_stem_match(
    token: &TokenInfo,
    index: usize,
    tokens: &[TokenInfo],
    native_language: &NativeLanguage,
) -> Option<GrammarMatch> {
    if token.part_of_speech() != &PartOfSpeech::Verb {
        return None;
    }
    if token.orthographic_surface_form() == token.orthographic_base_form() {
        return None;
    }
    // Honorific irregular verbs (為さる/下さる/いらっしゃる) have their own
    // suppletive rules (VerbToNasai/Kudasai/Irasshai) and must NOT fall back
    // to the continuative label.
    if HONORIFIC_VERB_BASES.contains(&token.orthographic_base_form()) {
        return None;
    }
    if next_token_is_conjugation_suffix(tokens, index) {
        return None;
    }
    let rules = GRAMMAR_RULES.get()?;
    let rule = rules
        .iter()
        .find(|r| r.content(native_language).title().contains("連用形"))?;
    Some(GrammarMatch::from_rule(rule, native_language))
}

const HONORIFIC_VERB_BASES: &[&str] = &[
    "為さる",
    "なさる",
    "下さる",
    "くださる",
    "御座る",
    "ござる",
    "いらっしゃる",
];

#[cfg(test)]
mod integration_tests {
    use super::super::lookup_tokens_translations;
    use super::super::test_env::ensure_dictionaries;
    use crate::domain::NativeLanguage;
    use crate::domain::tokenizer::tokenize_text;

    // A verb masu-stem standing on its own (next token NOT ます/て/た/ば/...)
    // is a continuative or a noun-ified stem. It must carry the 連用形
    // grammar_label.
    #[test]
    fn should_label_masu_stem_continuative_when_standalone() {
        ensure_dictionaries();
        let text = "返事を願いします";
        let tokens = tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, text);
        let negai = results
            .iter()
            .find(|t| t.surface_form == "願い")
            .expect("「願い」token should exist");
        assert!(
            negai
                .grammar_label
                .as_deref()
                .is_some_and(|l| l.contains("連用形")),
            "standalone masu-stem「願い」should carry the continuative grammar_label, got: {:?}",
            negai
        );
    }
    // Anti-regression: a masu-stem directly before ます (polite auxiliary) MUST
    // NOT carry the continuative label — it is the base of the polite form and
    // is covered by the ～ます rule.
    #[test]
    fn should_not_label_masu_stem_before_polite_aux() {
        ensure_dictionaries();
        let text = "ご飯を食べます";
        let tokens = tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, text);
        let tabe = results
            .iter()
            .find(|t| t.surface_form == "食べ")
            .expect("「食べ」stem token should exist");
        assert!(
            !tabe
                .grammar_label
                .as_deref()
                .is_some_and(|l| l.contains("連用形")),
            "「食べ」before 「ます」must NOT carry the continuative label (it is the polite base), got: {:?}",
            tabe
        );
    }
    // Anti-regression: masu-stem before て (te-form Particle) must not be
    // labeled continuative — it is the base of the te-form. This is the case
    // the POS verification gate flagged: て is Particle (72k) more often than
    // AuxiliaryVerb, so a POS-only guard would over-match.
    #[test]
    fn should_not_label_masu_stem_before_te_particle() {
        ensure_dictionaries();
        let text = "食べて飲んで";
        let tokens = tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, text);
        let tabe = results
            .iter()
            .find(|t| t.surface_form == "食べ")
            .expect("「食べ」stem token should exist");
        assert!(
            !tabe
                .grammar_label
                .as_deref()
                .is_some_and(|l| l.contains("連用形")),
            "「食べ」before te-form「て」must NOT carry the continuative label, got: {:?}",
            tabe
        );
    }
    // A standalone masu-stem whose surface exists as a noun in the vocabulary
    // dictionary (願い = request) must surface the noun translation rather than
    // the verb-base translation (願う = to request). The assertion compares
    // against `get_translation` directly so a regression that swaps surface
    // and base lookup is caught.
    #[test]
    fn should_use_noun_translation_for_standalone_masu_stem() {
        ensure_dictionaries();
        let text = "返事を願いします";
        let tokens = tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, text);
        let negai = results
            .iter()
            .find(|t| t.surface_form == "願い")
            .expect("「願い」token should exist");

        use crate::dictionary::vocabulary::get_translation;
        let surface_translation = get_translation("願い", &NativeLanguage::Russian);
        let base_translation = get_translation("願う", &NativeLanguage::Russian);
        // If the dictionary exposes distinct noun (surface) and verb (base)
        // entries, the noun entry must win for a standalone masu-stem.
        if surface_translation.is_some()
            && base_translation.is_some()
            && surface_translation != base_translation
        {
            assert_eq!(
                negai.translation, surface_translation,
                "standalone「願い」must use the surface (noun) translation, got: {:?}",
                negai.translation
            );
        } else {
            assert!(
                negai.translation.is_some(),
                "standalone「願い」should carry a translation, got: {:?}",
                negai
            );
        }
    }
    // 為さる → なさい (polite imperative). The suppletive form is produced by
    // the VerbToNasai FormatAction variant (matched by lemma only).
    #[test]
    fn should_label_nasai_imperative_from_nasaru() {
        ensure_dictionaries();
        let text = "よく考えなさい";
        let tokens = tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, text);
        let nasai = results
            .iter()
            .find(|t| t.surface_form == "なさい")
            .expect("「なさい」token should exist");
        assert!(
            nasai.grammar_label.is_some(),
            "「なさい」from 為さる should carry a grammar_label, got: {:?}",
            nasai
        );
    }
    // 下さる → ください (polite imperative). The suppletive form is produced by
    // the VerbToKudasai FormatAction variant.
    #[test]
    fn should_label_kudasai_imperative_from_kudasaru() {
        ensure_dictionaries();
        let text = "これをお食べください";
        let tokens = tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, text);
        let kudasai = results
            .iter()
            .find(|t| t.surface_form == "ください")
            .expect("「ください」token should exist");
        assert!(
            kudasai.grammar_label.is_some(),
            "「ください」from 下さる should carry a grammar_label, got: {:?}",
            kudasai
        );
    }
    // いらっしゃる → いらっしゃい (imperative greeting). The suppletive form is
    // produced by the VerbToIrasshai FormatAction variant.
    #[test]
    fn should_label_irasshai_imperative_from_irassharu() {
        ensure_dictionaries();
        let text = "いらっしゃいませ";
        let tokens = tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, text);
        let irasshai = results
            .iter()
            .find(|t| t.surface_form == "いらっしゃい")
            .expect("「いらっしゃい」token should exist");
        assert!(
            irasshai.grammar_label.is_some(),
            "「いらっしゃい」from いらっしゃる should carry a grammar_label, got: {:?}",
            irasshai
        );
    }
}
