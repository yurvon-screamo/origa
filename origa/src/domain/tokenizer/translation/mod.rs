//! Token translation pipeline.
//!
//! Maps tokenizer output to per-token translations and grammar labels:
//! each token's translation is resolved from the vocabulary dictionary
//! (with POS-specific fallback order), and its grammar label from the
//! resolvers in the sibling modules (`grammar_match`, `sou_da`,
//! `masu_stem`) plus the counter suffix table. Decomposed from the former
//! 2300-line translation.rs god-file (issue #796).

mod grammar_match;
mod masu_stem;
mod sou_da;

#[cfg(test)]
mod pipeline_tests;
#[cfg(test)]
mod test_env;

use serde::Serialize;

use grammar_match::{resolve_grammar_match, split_grammar_fields};
use masu_stem::resolve_masu_stem_match;
use sou_da::resolve_sou_da_match;

use super::{PartOfSpeech, TokenInfo};
use crate::dictionary::vocabulary::get_translation;
use crate::domain::NativeLanguage;
use crate::domain::katakana_to_hiragana;

#[derive(Debug, Clone, Serialize)]
pub struct TokenTranslation {
    pub surface_form: String,
    pub base_form: String,
    pub reading: String,
    pub pos: PartOfSpeech,
    pub translation: Option<String>,
    pub grammar_label: Option<String>,
    pub grammar_description: Option<String>,
}

// Particle homonyms whose surface form (て/し) collides with a dictionary lemma
// (手 "hand", 為る "to do"). Lindera surfaces base_form == surface_form for these
// particles, so the translation pipeline would otherwise resolve the unrelated
// content-word meaning instead of leaving the particle untranslated.
const PARTICLE_HOMONYM_SURFACES: &[&str] = &["て", "し"];

fn is_particle_homonym_blacklisted(token: &TokenInfo) -> bool {
    token.part_of_speech() == &PartOfSpeech::Particle
        && token.orthographic_base_form() == token.orthographic_surface_form()
        && PARTICLE_HOMONYM_SURFACES.contains(&token.orthographic_surface_form())
}

pub fn lookup_tokens_translations(
    tokens: &[TokenInfo],
    native_language: &NativeLanguage,
    original_text: &str,
) -> Vec<TokenTranslation> {
    // Computed once for the whole phrase: the katakana→hiragana equivalent of
    // the input lets `match_grammar_keyword` match katakana-written tokens
    // (e.g. ベキ) against hiragana grammar keywords (e.g. べき) without
    // re-normalizing the same text for every token and rule.
    let original_hiragana = katakana_to_hiragana(original_text);

    tokens
        .iter()
        .enumerate()
        .map(|(index, token)| {
            let base_form = token.orthographic_base_form().to_string();
            let surface_form = token.orthographic_surface_form().to_string();

            // The masu-stem resolver runs first because it also drives the
            // translation fallback: a standalone masu-stem (not followed by a
            // conjugation suffix) prefers its noun dictionary entry (surface)
            // over the verb lemma (base).
            let masu_stem_match = resolve_masu_stem_match(token, index, tokens, native_language);

            let translation = if is_particle_homonym_blacklisted(token) {
                None
            } else if masu_stem_match.is_some() {
                get_translation(&surface_form, native_language)
                    .or_else(|| get_translation(&base_form, native_language))
            } else if token.part_of_speech() == &PartOfSpeech::Interjection {
                // Interjection lemmas in the tokenizer dictionary frequently
                // disagree with the translation dictionary (うわー normalizes to
                // うわ, while the translation dictionary keys the full surface).
                // Prefer the surface spelling, then the lemma.
                get_translation(&surface_form, native_language)
                    .or_else(|| get_translation(&base_form, native_language))
            } else {
                get_translation(&base_form, native_language)
            };

            let grammar = resolve_sou_da_match(token, index, tokens, native_language)
                .or_else(|| {
                    resolve_grammar_match(token, native_language, original_text, &original_hiragana)
                })
                .or(masu_stem_match);
            let (mut grammar_label, mut grammar_description) = split_grammar_fields(grammar);

            // Counter-резолв — последняя позиция цепочки (реальные правила
            // грамматики 分/回/度/件 никогда не маскируются) и заполняет
            // метку только если она ещё пуста. POS-гейт «только в
            // контексте»: standalone 本 — Noun «книга» со словарным
            // переводом, метки суффикса не получает (issue #415).
            if grammar_label.is_none()
                && token.part_of_speech() == &PartOfSpeech::Suffix
                && let Some(entry) =
                    crate::dictionary::counters::get_counter(token.orthographic_surface_form())
            {
                grammar_label =
                    Some(crate::dictionary::counters::counter_label(*native_language).to_string());
                grammar_description = Some(
                    crate::dictionary::counters::gloss_for(entry, *native_language).to_string(),
                );
            }

            TokenTranslation {
                surface_form,
                base_form,
                reading: token.phonological_surface_form().to_string(),
                pos: token.part_of_speech().clone(),
                translation,
                grammar_label,
                grammar_description,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_token(base: &str, surface: &str, reading: &str, pos: PartOfSpeech) -> TokenInfo {
        TokenInfo {
            orthographic_base_form: base.to_string(),
            phonological_base_form: reading.to_string(),
            orthographic_surface_form: surface.to_string(),
            phonological_surface_form: reading.to_string(),
            part_of_speech: pos,
        }
    }
    #[test]
    fn should_map_all_fields_from_token_info() {
        let tokens = vec![TokenInfo {
            orthographic_base_form: "食べる".to_string(),
            phonological_base_form: "タベル".to_string(),
            orthographic_surface_form: "食べ".to_string(),
            phonological_surface_form: "タベ".to_string(),
            part_of_speech: PartOfSpeech::Verb,
        }];

        let result = lookup_tokens_translations(&tokens, &NativeLanguage::English, "");

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].base_form, "食べる");
        assert_eq!(result[0].surface_form, "食べ");
        assert_eq!(result[0].reading, "タベ");
        assert_eq!(result[0].pos, PartOfSpeech::Verb);
    }
    #[test]
    fn should_return_none_translation_for_unknown_word() {
        let tokens = vec![make_token("未知語", "未知語", "ミチゴ", PartOfSpeech::Noun)];

        let result = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, "");

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].base_form, "未知語");
        assert_eq!(result[0].reading, "ミチゴ");
        assert!(result[0].translation.is_none());
    }
    #[test]
    fn should_include_punctuation_with_none_translation() {
        let tokens = vec![make_token("。", "。", "。", PartOfSpeech::Symbol)];

        let result = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, "");

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].surface_form, "。");
        assert!(result[0].translation.is_none());
    }
    #[test]
    fn should_handle_multiple_tokens() {
        let tokens = vec![
            make_token("猫", "猫", "ネコ", PartOfSpeech::Noun),
            make_token("は", "は", "ハ", PartOfSpeech::Particle),
            make_token("。", "。", "。", PartOfSpeech::Symbol),
        ];

        let result = lookup_tokens_translations(&tokens, &NativeLanguage::English, "");

        assert_eq!(result.len(), 3);
        assert_eq!(result[0].base_form, "猫");
        assert_eq!(result[1].base_form, "は");
        assert_eq!(result[2].surface_form, "。");
    }
    #[test]
    fn should_return_empty_for_empty_input() {
        let tokens: Vec<TokenInfo> = vec![];

        let result = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, "");

        assert!(result.is_empty());
    }
    #[test]
    fn should_apply_homonym_blacklist_for_particle_te_and_shi() {
        assert!(is_particle_homonym_blacklisted(&make_token(
            "て",
            "て",
            "テ",
            PartOfSpeech::Particle
        )));
        assert!(is_particle_homonym_blacklisted(&make_token(
            "し",
            "し",
            "シ",
            PartOfSpeech::Particle
        )));

        // Topic は particle is unaffected (no dictionary homonym in this list).
        assert!(!is_particle_homonym_blacklisted(&make_token(
            "は",
            "は",
            "ハ",
            PartOfSpeech::Particle
        )));
        // Same surface form but non-Particle POS must not match.
        assert!(!is_particle_homonym_blacklisted(&make_token(
            "て",
            "て",
            "テ",
            PartOfSpeech::Noun
        )));
        assert!(!is_particle_homonym_blacklisted(&make_token(
            "し",
            "し",
            "シ",
            PartOfSpeech::Verb
        )));
        assert!(!is_particle_homonym_blacklisted(&make_token(
            "し",
            "し",
            "シ",
            PartOfSpeech::AuxiliaryVerb
        )));
        // Particle て surface != base (conjugated verb てる) must not match.
        assert!(!is_particle_homonym_blacklisted(&make_token(
            "て",
            "てる",
            "テ",
            PartOfSpeech::Particle
        )));
    }
}
