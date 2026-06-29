use serde::Serialize;

use super::{PartOfSpeech, TokenInfo};
use crate::dictionary::grammar::{GRAMMAR_RULES, GrammarRule};
use crate::dictionary::vocabulary::get_translation;
use crate::domain::JapaneseChar;
use crate::domain::NativeLanguage;
use crate::domain::grammar::find_format_map_matches;
use crate::domain::katakana_to_hiragana;

#[derive(Debug, Clone, Serialize)]
pub struct TokenTranslation {
    pub surface_form: String,
    pub base_form: String,
    pub reading: String,
    pub pos: PartOfSpeech,
    pub translation: Option<String>,
    pub grammar_label: Option<String>,
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
    tokens
        .iter()
        .enumerate()
        .map(|(index, token)| {
            let base_form = token.orthographic_base_form().to_string();
            let translation = if is_particle_homonym_blacklisted(token) {
                None
            } else {
                get_translation(&base_form, native_language)
            };

            let grammar_label = resolve_sou_da_label(token, index, tokens, native_language)
                .or_else(|| resolve_grammar_label(token, native_language, original_text));

            TokenTranslation {
                surface_form: token.orthographic_surface_form().to_string(),
                base_form,
                reading: token.phonological_surface_form().to_string(),
                pos: token.part_of_speech().clone(),
                translation,
                grammar_label,
            }
        })
        .collect()
}

fn match_grammar_keyword(
    rules: &[GrammarRule],
    surface: &str,
    original_text: &str,
    native_language: &NativeLanguage,
) -> Option<String> {
    for rule in rules.iter() {
        let keyword_groups = rule.keywords();
        if keyword_groups.is_empty() {
            continue;
        }

        let token_matches_some_group = keyword_groups
            .iter()
            .any(|group| group.iter().any(|kw| surface == kw));
        if !token_matches_some_group {
            continue;
        }

        // Every group must contribute at least one keyword present in the original
        // text. This converges with detect_keyword_rules AND-semantics and prevents
        // bare-single-keyword false positives (e.g. topic は falsely matching the
        // ～は～ほど～ない rule when ほど is absent from the sentence).
        let text_covers_all_groups = keyword_groups
            .iter()
            .all(|group| !group.is_empty() && group.iter().any(|kw| original_text.contains(kw)));
        if text_covers_all_groups {
            return Some(rule.content(native_language).title().to_string());
        }
    }
    None
}

fn resolve_grammar_label(
    token: &TokenInfo,
    native_language: &NativeLanguage,
    original_text: &str,
) -> Option<String> {
    let rules = GRAMMAR_RULES.get()?;
    let surface = token.orthographic_surface_form();
    let base = token.orthographic_base_form();
    let pos = token.part_of_speech();
    let is_vocab = pos.is_vocabulary_word();

    // Keyword matching for non-vocabulary tokens (particles, auxiliaries, etc.)
    // — and for vocabulary tokens whose surface form equals the dictionary
    // base form AND matches a grammar keyword. The dictionary-form gate
    // prevents conjugated content words (e.g. 食べた, 食べて) from getting a
    // grammar_label via keyword lookup while still recognizing grammar nouns
    // like べき, はず, ところ that Lindera classifies as Noun.
    // See issue #178 P-6 for context.
    let eligible_for_keyword_match = !is_vocab || surface == base;
    if eligible_for_keyword_match {
        if let Some(label) = match_grammar_keyword(rules, surface, original_text, native_language) {
            return Some(label);
        }
    }

    if is_vocab && surface != base {
        // Conjugated auxiliaries attached to a content stem (e.g. すぎ inside 強すぎて,
        // base すぎる, classified by Lindera as Verb) must still resolve to their
        // grammar keyword before format_map matching — otherwise a more generic
        // rule (～て) wins on the te-form surface.
        if let Some(label) = match_grammar_keyword(rules, surface, original_text, native_language) {
            return Some(label);
        }

        let mut matches = find_format_map_matches(base, pos, original_text, rules);

        // Lindera prefers the kanji lemma as ``base`` even when the user's
        // phrase is written entirely in hiragana. ``find_format_map_matches``
        // formats the base into its conjugated surface (e.g. ``分かる`` →
        // ``分かって``) and checks the original text — but the kanji-formatted
        // string never appears in a hiragana phrase, so the match silently
        // fails. Retry the same chain with the hiragana-equivalent base
        // derived from the katakana reading so ``分かる`` becomes ``わかる`` and
        // the formatted ``わかって`` does occur inside ``わかってるなら``.
        let hiragana_base = if base.chars().any(|c| c.is_kanji()) {
            Some(katakana_to_hiragana(token.phonological_base_form()))
        } else {
            None
        };

        if let Some(ref hira) = hiragana_base {
            matches.extend(find_format_map_matches(hira, pos, original_text, rules));
        }

        // Score each candidate by the longest format it produces across both
        // bases (kanji + hiragana-equivalent) so rules that only match through
        // the hiragana retry are not penalised when ``rule.format(base, pos)``
        // happens to error on the kanji lemma.
        if let Some(best) = matches.into_iter().max_by_key(|rule| {
            let kanji_len = rule.format(base, pos).map_or(0, |f| f.len());
            let hira_len = hiragana_base
                .as_ref()
                .map(|h| rule.format(h, pos).map_or(0, |f| f.len()))
                .unwrap_or(0);
            kanji_len.max(hira_len)
        }) {
            return Some(best.content(native_language).title().to_string());
        }
    }

    None
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SouDaVariant {
    Appearance,
    Hearsay,
    Combined,
}

/// Context-aware detection for the そうだ grammar pattern.
///
/// そう has two distinct meanings depending on what precedes it:
/// - 様態 (appearance/conjecture): follows a verb/adjective STEM form
///   (美味し→美味しそう, 降り→降りそう). Previous token: conjugated (surface != base).
/// - 伝聞 (hearsay): follows a PLAIN/dictionary form (食べる→食べるそうだ) or a
///   past/copula auxiliary (降った→降ったそうだ, 元気だ→元気だそうだ).
///
/// A standalone そう that is NOT part of a そうだ construction (the demonstrative
/// manner adverb そう in そう思う, そうですね) must be left to `resolve_grammar_label`
/// so the manner rule `こう・そう・ああ・どう` can match — this function returns
/// `None` whenever the preceding token is not a predicate the pattern could
/// attach to (particle, adverb, pronoun, ...) or when a sentence boundary
/// separates そう from any predecessor.
fn resolve_sou_da_label(
    token: &TokenInfo,
    index: usize,
    tokens: &[TokenInfo],
    native_language: &NativeLanguage,
) -> Option<String> {
    let surface = token.orthographic_surface_form();

    // Lindera splits the grammar pattern into a standalone そう token followed
    // by a copula (だ/です). Compound words like そういう / そうした are distinct
    // lemmas and must not trigger this path.
    if surface != "そう" {
        return None;
    }

    // Exclude content lemmas whose base form carries kanji (e.g. 添う "to comply",
    // surfaced as そう) — those are dictionary verbs, not the grammar pattern.
    // The grammar-pattern そう surfaces base そう with no kanji.
    let pos = token.part_of_speech();
    if pos.is_vocabulary_word() && token.orthographic_base_form().chars().any(|c| c.is_kanji()) {
        return None;
    }

    if index == 0 {
        return None;
    }

    let prev = previous_predicate(tokens, index)?;

    let prev_surface = prev.orthographic_surface_form();
    let prev_base = prev.orthographic_base_form();
    let prev_pos = prev.part_of_speech();

    let is_stem_form = prev_surface != prev_base
        && matches!(prev_pos, PartOfSpeech::Verb | PartOfSpeech::IAdjective);

    let is_plain_form = prev_surface == prev_base
        && matches!(prev_pos, PartOfSpeech::Verb | PartOfSpeech::IAdjective);

    // た (past) and だ (copula) auxiliaries both attach そう as hearsay:
    // 降ったそうだ (past), 元気だそうだ / 学生だそうだ (copula plain form).
    let is_ta_or_da_aux = matches!(prev_pos, PartOfSpeech::AuxiliaryVerb)
        && (prev_surface == "た" || prev_surface == "だ");

    // A na-adjective/noun directly before そう (元気そう, 静かそう) is token-ambiguous:
    // the 様態 stem and the dictionary form are visually identical. Noun/na-adj
    // hearsay always carries an intervening copula だ (caught by is_ta_or_da_aux),
    // so a bare noun/na-adj predecessor leans appearance — the combined rule
    // covers both readings without forcing a wrong specific label.
    let is_ambiguous_stem = matches!(
        prev_pos,
        PartOfSpeech::NaAdjective | PartOfSpeech::Noun | PartOfSpeech::ProperNoun
    );

    let rules = GRAMMAR_RULES.get()?;

    if is_stem_form {
        find_sou_rule(rules, native_language, SouDaVariant::Appearance)
    } else if is_plain_form || is_ta_or_da_aux {
        find_sou_rule(rules, native_language, SouDaVariant::Hearsay)
    } else if is_ambiguous_stem {
        find_sou_rule(rules, native_language, SouDaVariant::Combined)
    } else {
        None
    }
}

/// Locates the nearest content token preceding `index`, stopping at a sentence
/// boundary. Returns `None` when a sentence-ending punctuation (。！？) is hit
/// before any content token, so そう starting a new clause (食べた。そうだ。) is
/// not misread as hearsay about the previous sentence.
fn previous_predicate(tokens: &[TokenInfo], index: usize) -> Option<&TokenInfo> {
    for t in tokens[..index].iter().rev() {
        if is_sentence_boundary(t) {
            return None;
        }
        if matches!(
            t.part_of_speech(),
            PartOfSpeech::Whitespace | PartOfSpeech::Symbol | PartOfSpeech::AuxiliarySymbol
        ) {
            continue;
        }
        return Some(t);
    }
    None
}

fn is_sentence_boundary(token: &TokenInfo) -> bool {
    matches!(
        token.part_of_speech(),
        PartOfSpeech::Symbol | PartOfSpeech::AuxiliarySymbol
    ) && token
        .orthographic_surface_form()
        .chars()
        .any(|c| matches!(c, '。' | '！' | '？'))
}

/// Resolves a そうだ grammar rule title for the requested variant. Terms are
/// anchored to `そうだ（…）` so the hearsay search does not collide with
/// unrelated hearsay rules whose titles also contain `（伝聞）` (～という（伝聞）,
/// ～とか（伝聞）). Falls back to any `そうだ` rule when the dictionary lacks a
/// variant-specific entry.
fn find_sou_rule(
    rules: &[GrammarRule],
    native_language: &NativeLanguage,
    variant: SouDaVariant,
) -> Option<String> {
    let term_groups: &[&[&str]] = match variant {
        SouDaVariant::Appearance => &[&["そうだ（様態）"], &["そうだ"]],
        SouDaVariant::Hearsay => &[&["そうだ（伝聞）"], &["そうだ"]],
        SouDaVariant::Combined => &[&["そうだ（様態・伝聞）"], &["そうだ"]],
    };

    for terms in term_groups {
        for rule in rules.iter() {
            let title = rule.content(native_language).title();
            if terms.iter().any(|term| title.contains(term)) {
                return Some(title.to_string());
            }
        }
    }
    None
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

#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::dictionary::grammar::{GrammarData, init_grammar, is_grammar_loaded};
    use crate::dictionary::vocabulary::{
        VocabularyChunkData, init_vocabulary, is_vocabulary_loaded,
    };
    use crate::domain::{DictionaryData, init_dictionary, is_dictionary_loaded};

    fn ensure_dictionaries() {
        ensure_tokenizer_dictionary();
        ensure_vocabulary_dictionary();
        ensure_grammar_dictionary();
    }

    fn ensure_tokenizer_dictionary() {
        if is_dictionary_loaded() {
            return;
        }

        use flate2::read::DeflateDecoder;
        use std::fs;
        use std::io::Read;

        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let dict_dir = std::path::PathBuf::from(manifest_dir)
            .parent()
            .unwrap()
            .join("cdn")
            .join("dictionaries");

        let decompress = |data: Vec<u8>| -> Vec<u8> {
            let mut decoder = DeflateDecoder::new(&data[..]);
            let mut decompressed = Vec::new();
            decoder.read_to_end(&mut decompressed).unwrap();
            decompressed
        };

        let read_file = |name: &str| fs::read(dict_dir.join(name)).unwrap();

        let data = DictionaryData {
            char_def: decompress(read_file("char_def.bin")),
            matrix: decompress(read_file("matrix.mtx")),
            dict_da: decompress(read_file("dict.da")),
            dict_vals: decompress(read_file("dict.vals")),
            unk: decompress(read_file("unk.bin")),
            words_idx: decompress(read_file("dict.wordsidx")),
            words: decompress(read_file("dict.words")),
            metadata: read_file("metadata.json"),
        };

        init_dictionary(data).unwrap();
    }

    fn ensure_vocabulary_dictionary() {
        if is_vocabulary_loaded() {
            return;
        }

        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let vocab_dir = std::path::PathBuf::from(manifest_dir)
            .parent()
            .unwrap()
            .join("cdn")
            .join("dictionary");

        let read_chunk = |name: &str| std::fs::read_to_string(vocab_dir.join(name)).unwrap();

        let vocab_data = VocabularyChunkData {
            chunk_01: read_chunk("chunk_01.json"),
            chunk_02: read_chunk("chunk_02.json"),
            chunk_03: read_chunk("chunk_03.json"),
            chunk_04: read_chunk("chunk_04.json"),
            chunk_05: read_chunk("chunk_05.json"),
            chunk_06: read_chunk("chunk_06.json"),
            chunk_07: read_chunk("chunk_07.json"),
            chunk_08: read_chunk("chunk_08.json"),
            chunk_09: read_chunk("chunk_09.json"),
            chunk_10: read_chunk("chunk_10.json"),
            chunk_11: read_chunk("chunk_11.json"),
        };

        // Tolerate the OnceLock init race: a concurrent test thread may have
        // initialized the vocabulary first with identical data. Only a genuinely
        // unset dictionary indicates a real failure; preserve the error detail
        // so data regressions stay debuggable.
        if let Err(e) = init_vocabulary(vocab_data) {
            if !is_vocabulary_loaded() {
                panic!("vocabulary dictionary failed to load: {:?}", e);
            }
        }
    }

    fn ensure_grammar_dictionary() {
        if is_grammar_loaded() {
            return;
        }

        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let grammar_path = std::path::PathBuf::from(manifest_dir)
            .parent()
            .unwrap()
            .join("cdn")
            .join("grammar")
            .join("grammar.json");

        let grammar_json = std::fs::read_to_string(grammar_path).unwrap();
        // Same OnceLock race tolerance as ensure_vocabulary_dictionary.
        if let Err(e) = init_grammar(GrammarData { grammar_json }) {
            if !is_grammar_loaded() {
                panic!("grammar dictionary failed to load: {:?}", e);
            }
        }
    }

    #[test]
    fn should_translate_bakari() {
        ensure_dictionaries();
        let text = "ばかり";
        let tokens = super::super::tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, text);
        let bakari = results.iter().find(|t| t.surface_form.contains("ばかり"));
        assert!(bakari.is_some(), "「ばかり」token should exist");
        assert!(
            bakari.unwrap().translation.is_some(),
            "「ばかり」should have a translation"
        );
    }

    #[test]
    fn should_translate_uwa_interjection() {
        ensure_dictionaries();
        let text = "うわー";
        let tokens = super::super::tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, text);
        let uwa = results.iter().find(|t| t.surface_form.contains("うわ"));
        assert!(uwa.is_some(), "「うわー」token should exist");
        assert!(
            uwa.unwrap().translation.is_some(),
            "「うわー」should have a translation"
        );
    }

    #[test]
    fn should_translate_souiu_compound() {
        ensure_dictionaries();
        let text = "そういう";
        let tokens = super::super::tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, text);
        let souiu = results.iter().find(|t| t.surface_form.contains("そういう"));
        assert!(souiu.is_some(), "「そういう」token should exist");
        assert!(
            souiu.unwrap().translation.is_some(),
            "「そういう」should have a translation"
        );
    }

    #[test]
    fn should_translate_hodo() {
        ensure_dictionaries();
        let text = "ほど";
        let tokens = super::super::tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, text);
        let hodo = results.iter().find(|t| t.surface_form.contains("ほど"));
        assert!(hodo.is_some(), "「ほど」token should exist");
        assert!(
            hodo.unwrap().translation.is_some(),
            "「ほど」should have a translation"
        );
    }

    #[test]
    fn should_resolve_grammar_label_for_particle() {
        ensure_dictionaries();
        let text = "東京から大阪まで";
        let tokens = super::super::tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::English, text);
        let kara_particle = results.iter().find(|t| t.surface_form == "から");
        assert!(
            kara_particle.is_some(),
            "「から」particle should exist in tokens"
        );
        let kara = kara_particle.unwrap();
        assert!(
            kara.grammar_label.is_some(),
            "「から」should have a grammar label, got: {:?}",
            kara
        );
    }

    #[test]
    fn should_detect_te_form_grammar_label() {
        ensure_dictionaries();
        let text = "食べて";
        let tokens = super::super::tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::English, text);
        let verb_token = results.iter().find(|t| t.base_form == "食べる");
        assert!(verb_token.is_some(), "Should find 食べる token");
        let verb = verb_token.unwrap();
        assert!(
            verb.grammar_label.is_some(),
            "「食べて」verb should have grammar_label for te-form, got: {:?}",
            verb
        );
    }

    #[test]
    fn should_detect_tai_form_grammar_label() {
        ensure_dictionaries();
        let text = "食べたい";
        let tokens = super::super::tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::English, text);
        let verb_token = results.iter().find(|t| t.base_form == "食べる");
        assert!(verb_token.is_some(), "Should find 食べる token");
        let verb = verb_token.unwrap();
        assert!(
            verb.grammar_label.is_some(),
            "「食べたい」verb should have grammar_label for tai-form, got: {:?}",
            verb
        );
    }

    #[test]
    fn should_detect_ta_form_grammar_label() {
        ensure_dictionaries();
        let text = "食べた";
        let tokens = super::super::tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::English, text);
        let verb_token = results.iter().find(|t| t.base_form == "食べる");
        assert!(verb_token.is_some(), "Should find 食べる token");
        let verb = verb_token.unwrap();
        assert!(
            verb.grammar_label.is_some(),
            "「食べた」verb should have grammar_label for ta-form, got: {:?}",
            verb
        );
    }

    #[test]
    fn should_detect_nai_form_grammar_label() {
        ensure_dictionaries();
        let text = "食べない";
        let tokens = super::super::tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::English, text);
        let verb_token = results.iter().find(|t| t.base_form == "食べる");
        assert!(verb_token.is_some(), "Should find 食べる token");
        let verb = verb_token.unwrap();
        assert!(
            verb.grammar_label.is_some(),
            "「食べない」verb should have grammar_label for nai-form, got: {:?}",
            verb
        );
    }

    #[test]
    fn should_not_set_grammar_label_for_base_form() {
        ensure_dictionaries();
        let text = "食べる";
        let tokens = super::super::tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::English, text);
        let verb_token = results.iter().find(|t| t.base_form == "食べる");
        assert!(verb_token.is_some(), "Should find 食べる token");
        assert!(
            verb_token.unwrap().grammar_label.is_none(),
            "Base form should NOT have grammar_label"
        );
    }

    #[test]
    fn should_detect_beki_grammar_label() {
        ensure_dictionaries();
        let text = "うん、あるべきものをあるべき形カタチに";
        let tokens = super::super::tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::English, text);

        let beki = results.iter().find(|t| t.surface_form == "べき");
        assert!(
            beki.is_some(),
            "「べき」token should exist, tokens: {:?}",
            results.iter().map(|t| &t.surface_form).collect::<Vec<_>>()
        );
        let beki = beki.unwrap();
        assert!(
            beki.grammar_label
                .as_ref()
                .is_some_and(|label| label.contains("べき")),
            "「べき」should carry the べきだ grammar_label, got: {:?}",
            beki
        );
    }

    // Regression for issue #178 P-6: a standalone べき token is classified
    // by Lindera as Noun (助動詞語幹). The old `!is_vocab` gate skipped keyword
    // matching entirely for Noun tokens, so べき never received a grammar_label.
    // The fix allows keyword lookup when surface == base so dictionary-form
    // grammar nouns (べき, はず, ところ, etc.) still resolve to a grammar rule.
    #[test]
    fn tokenize_standalone_beki_returns_grammar_label() {
        ensure_dictionaries();
        let text = "べき";
        let tokens = super::super::tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::English, text);

        let beki = results
            .iter()
            .find(|t| t.surface_form == "べき")
            .expect("「べき」standalone token should exist");
        assert!(
            beki.grammar_label
                .as_ref()
                .is_some_and(|label| label.contains("べき")),
            "standalone「べき」should carry the べきだ grammar_label, got: {:?}",
            beki
        );
    }

    // Negative test for the relaxed grammar_label gate: a common content
    // noun in dictionary form (猫, "cat") must NOT pick up a spurious
    // grammar_label just because surface == base. Guards against the
    // relaxation over-matching content vocabulary that happens to collide
    // with a grammar keyword.
    #[test]
    fn tokenize_common_content_noun_has_no_grammar_label() {
        ensure_dictionaries();
        let text = "猫";
        let tokens = super::super::tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::English, text);

        let neko = results
            .iter()
            .find(|t| t.surface_form == "猫")
            .expect("「猫」token should exist");
        assert!(
            neko.grammar_label.is_none(),
            "common noun「猫」must not receive a spurious grammar_label, got: {:?}",
            neko
        );
    }

    // Verification for issue #178 P-8: characterization of how Lindera tokenizes
    // "さあついたよ、ここだ". Result is correct as-is (interjection + verb past +
    // particle + punct + pronoun + copula) — wontfix with this note.
    #[test]
    fn tokenize_saa_tsuita_yo_kokoda_returns_expected_tokens() {
        ensure_dictionaries();
        let text = "さあついたよ、ここだ";
        let tokens = super::super::tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::English, text);

        let surfaces: Vec<&str> = results.iter().map(|t| t.surface_form.as_str()).collect();
        assert!(
            surfaces.contains(&"さあ"),
            "「さあ」interjection should be tokenized separately, got: {:?}",
            surfaces
        );
        assert!(
            surfaces
                .iter()
                .any(|s| s.contains("ついた") || s.contains("つい")),
            "「ついた」verb past should be tokenized, got: {:?}",
            surfaces
        );
        assert!(
            surfaces.contains(&"よ"),
            "「よ」particle should be tokenized, got: {:?}",
            surfaces
        );
        assert!(
            surfaces.contains(&"ここ"),
            "「ここ」pronoun should be tokenized, got: {:?}",
            surfaces
        );
        assert!(
            surfaces.contains(&"だ"),
            "「だ」copula should be tokenized, got: {:?}",
            surfaces
        );
    }

    // --- Reproduction / regression tests for translation bugs ---
    // These tests were originally written as failing repro-tests to document
    // concrete root causes across 10 user-reported phrases. After the 6-slice
    // fix they now pass and serve as regression locks. The
    // `should_split_mizunashi_compound` test locks the post-processing
    // splitter that resolves the 水なし over-merge limitation.

    // --- Phrase 1: 人はパンのみで生きるにあらずですわよ ---
    // Before the fix, Lindera split the classical negative あらず into あら
    // (Verb, base 有る) + ず (AuxiliaryVerb), and the ず token surfaced base_form
    // 'ぬ' (Lindera's lemma), which was absent from the vocabulary dictionary —
    // so translation was None and no grammar rule labeled the ず classical-negative
    // auxiliary. This test locks in the vocabulary entry for ぬ that resolves it.
    #[test]
    fn should_label_or_translate_classical_negative_zu_auxiliary() {
        ensure_dictionaries();
        let text = "人はパンのみで生きるにあらずですわよ";
        let tokens = super::super::tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, text);

        let zu = results
            .iter()
            .find(|t| t.surface_form == "ず")
            .expect("「ず」auxiliary token should exist");
        assert!(
            zu.translation.is_some() || zu.grammar_label.is_some(),
            "classical negative 「ず」should carry a translation or grammar_label, got: {:?}",
            zu
        );
    }

    // --- Phrase 1: わ (sentence-final particle, feminine/emphatic) ---
    // Before the fix, the standalone 「わ」 particle had no vocabulary entry, so
    // translation was None. This test locks in the added translation.
    #[test]
    fn should_translate_wa_sentence_final_particle() {
        ensure_dictionaries();
        let text = "生きるにあらずですわよ";
        let tokens = super::super::tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, text);

        let wa = results
            .iter()
            .find(|t| t.surface_form == "わ" && t.pos == PartOfSpeech::Particle)
            .expect("「わ」particle token should exist");
        assert!(
            wa.translation.is_some(),
            "sentence-final 「わ」particle should have a translation, got: {:?}",
            wa
        );
    }

    // --- Phrase 2: 水なしでは生きていけない ---
    // Regression test for the post-processing splitter. Lindera over-merges
    // 水なし ("without water") into a single ProperNoun token (reading ミズナシ)
    // with no vocabulary translation. The splitter in `tokenize_text` detects
    // ProperNoun tokens ending in a known productive suffix (currently なし)
    // and splits them into separate Noun tokens so each gets translated.
    // Previous approach (user_dictionary.csv + なし) broke E2E via furigana
    // rendering; this code-only fix avoids touching the build binary.
    #[test]
    fn should_split_mizunashi_compound() {
        ensure_dictionaries();
        let text = "水なしでは生きていけない";
        let tokens = super::super::tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, text);

        let merged = results.iter().find(|t| t.surface_form == "水なし");
        assert!(
            merged.is_none(),
            "「水なし」should be split into separate tokens, got merged token: {:?}",
            merged
        );

        let mizu = results
            .iter()
            .find(|t| t.surface_form == "水" && t.pos == PartOfSpeech::Noun)
            .expect("「水」noun should exist after split");
        // Split must expose 水's dictionary entry — the whole point of the fix.
        assert!(
            mizu.translation.is_some(),
            "「水」should carry a translation after split, got: {:?}",
            mizu
        );
        // Reading is derived from the merged token's phonological form
        // (ミズナシ → strip suffix-length chars → ミズ), not the surface kanji.
        assert_ne!(
            mizu.reading, "水",
            "「水」reading must be katakana ミズ, not surface kanji"
        );

        // なし has no standalone dictionary entry in the current vocabulary —
        // the split still produces the token (needed for furigana/glossary
        // alignment), but translation absence is a dictionary-data concern,
        // not a splitter concern.
        let nashi = results
            .iter()
            .find(|t| t.surface_form == "なし")
            .expect("「なし」token should exist after split");
        assert_ne!(
            nashi.reading, "なし",
            "「なし」reading must be katakana ナシ derived from merged token"
        );
    }

    // --- Phrase 3: 大丈夫でやがる ---
    // Before the fix, やがる (derogatory auxiliary "to do, damn it") had a
    // vocabulary entry but the 〜てやがる / 〜でやがる construction was never flagged
    // with a grammar_label. This test locks in the grammar rule that labels it.
    #[test]
    fn should_label_yagaru_derogatory_auxiliary_grammar() {
        ensure_dictionaries();
        let text = "大丈夫でやがる";
        let tokens = super::super::tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, text);

        let yagaru = results
            .iter()
            .find(|t| t.surface_form == "やがる")
            .expect("「やがる」auxiliary token should exist");
        assert!(
            yagaru.grammar_label.is_some(),
            "〜やがる derogatory construction should carry a grammar_label, got: {:?}",
            yagaru
        );
    }

    // --- Phrase 4: 二度も言わんでええの！ ---
    // Before the fix, the colloquial negative 「ん」 (contraction of ぬ) was
    // tokenized with base_form 'ぬ'. Translation lookup went through base_form,
    // and 'ぬ' was absent from the vocabulary dictionary, so translation was None
    // even though 「ん」 itself was present. No grammar rule labeled the ん negative
    // either. The same ん/ぬ root cause recurred in phrase 7 (くれませんか). This
    // test locks in the ぬ vocabulary entry that resolves it.
    #[test]
    fn should_label_or_translate_n_negative_auxiliary() {
        ensure_dictionaries();
        let text = "二度も言わんでええの！";
        let tokens = super::super::tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, text);

        let n = results
            .iter()
            .find(|t| t.surface_form == "ん" && t.pos == PartOfSpeech::AuxiliaryVerb)
            .expect("「ん」auxiliary token should exist");
        assert!(
            n.translation.is_some() || n.grammar_label.is_some(),
            "negative 「ん」auxiliary should carry a translation or grammar_label, got base={:?} translation={:?} grammar={:?}",
            n.base_form,
            n.translation,
            n.grammar_label
        );
    }

    // --- Phrase 5: 雨の音強すぎて、テレビの音聞こえない ---
    // Before the fix, the 〜すぎる ("too much") grammar rule only matched via
    // format_map (強い → 強すぎる), but the text contained the te-form 強すぎて,
    // which format_map never produces — so the "too much" meaning was never
    // detected on either the 強 stem or the すぎ token. This test locks in the
    // keyword-based すぎる detection that resolves conjugated auxiliaries.
    #[test]
    fn should_detect_sugiru_too_much_for_tsuyosugite() {
        ensure_dictionaries();
        let text = "雨の音強すぎて、テレビの音聞こえない";
        let tokens = super::super::tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, text);

        let has_sugiru_label = results.iter().any(|t| {
            t.grammar_label
                .as_deref()
                .is_some_and(|label| label.contains("すぎる"))
        });
        assert!(
            has_sugiru_label,
            "〜すぎる (too much) should be detected for 強すぎて, labels: {:?}",
            results
                .iter()
                .filter(|t| t.grammar_label.is_some())
                .map(|t| (&t.surface_form, &t.grammar_label))
                .collect::<Vec<_>>()
        );
    }

    // --- Phrase 6: カッコイイの！ ---
    // Before the fix, the katakana adjective カッコイイ ("cool") was split into
    // カッコ (格好) + イイ (良い), so the single-word meaning "cool/stylish" was
    // lost. This test locks in the user-dictionary entry that keeps it one token.
    #[test]
    fn should_tokenize_kakkoii_katakana_as_single_token() {
        ensure_dictionaries();
        let text = "カッコイイの！";
        let tokens = super::super::tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, text);

        let kakkoii = results.iter().find(|t| t.surface_form == "カッコイイ");
        assert!(
            kakkoii.is_some(),
            "「カッコイイ」should be a single token, got surfaces: {:?}",
            results.iter().map(|t| &t.surface_form).collect::<Vec<_>>()
        );
    }

    // --- Phrase 7: かっこいいですね ---
    // Before the fix, the hiragana adjective かっこいい WAS present in the
    // vocabulary dictionary (chunk_11), but Lindera split it into かっこ (base 格好)
    // + いい (base 良い). Translation lookup used base_form, so the existing
    // かっこいい entry was never consulted and the "cool/stylish" meaning was lost.
    // This test locks in the user-dictionary entry that keeps it one token.
    #[test]
    fn should_use_kakkoii_dictionary_entry_for_hiragana_compound() {
        ensure_dictionaries();
        let text = "かっこいいですね";
        let tokens = super::super::tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, text);

        let uses_entry = results
            .iter()
            .any(|t| t.surface_form == "かっこいい" || t.base_form == "かっこいい");
        assert!(
            uses_entry,
            "「かっこいい」should resolve to its dictionary entry, got (surface, base): {:?}",
            results
                .iter()
                .map(|t| (&t.surface_form, &t.base_form))
                .collect::<Vec<_>>()
        );
    }

    // --- Phrase 8: 心をひとつに ---
    // Before the fix, ひとつ ("one") had a vocabulary entry but was split into
    // ひと (base 一) + つ (suffix), so the single-word entry was never used.
    // This test locks in the user-dictionary entry that keeps it one token.
    #[test]
    fn should_tokenize_hitotsu_as_single_token() {
        ensure_dictionaries();
        let text = "心をひとつに";
        let tokens = super::super::tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, text);

        let hitotsu = results
            .iter()
            .any(|t| t.surface_form == "ひとつ" || t.surface_form == "一つ");
        assert!(
            hitotsu,
            "「ひとつ」should be a single token, got surfaces: {:?}",
            results.iter().map(|t| &t.surface_form).collect::<Vec<_>>()
        );
    }

    // --- Phrase 9: やれやれ、無駄なことだよ ---
    // Before the fix, やれやれ ("good grief") had a vocabulary entry but was split
    // into two やれ tokens, so the interjection entry was never used and each half
    // was translated with the imperative meaning instead. This test locks in the
    // user-dictionary entry that keeps it one token.
    #[test]
    fn should_tokenize_yareyare_as_single_token() {
        ensure_dictionaries();
        let text = "やれやれ、無駄なことだよ。";
        let tokens = super::super::tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, text);

        let yareyare = results.iter().any(|t| t.surface_form == "やれやれ");
        assert!(
            yareyare,
            "「やれやれ」should be a single token, got surfaces: {:?}",
            results.iter().map(|t| &t.surface_form).collect::<Vec<_>>()
        );
    }

    // --- Phrase 9: 起きるものか ---
    // Before the fix, 〜ものか ("no way / absolutely not", rhetorical exclamation)
    // was present in the vocabulary dictionary (ru: "ни в коем случае!"), but
    // Lindera split ものか into もの (base 物) + か, so the rhetorical-meaning entry
    // was never consulted and もの resolved to the unrelated "thing" translation
    // instead. This test locks in the user-dictionary entry that keeps it one token.
    #[test]
    fn should_tokenize_monoka_as_single_token() {
        ensure_dictionaries();
        let text = "起きるものか";
        let tokens = super::super::tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, text);

        let monoka = results.iter().any(|t| t.surface_form == "ものか");
        assert!(
            monoka,
            "「ものか」should be a single token so its dictionary entry is used, got surfaces: {:?}",
            results.iter().map(|t| &t.surface_form).collect::<Vec<_>>()
        );
    }

    // --- Phrase 10: ふふ、ストーブついてますし ---
    // Before the fix, the interjection ふふ ("hehe") had no vocabulary entry, so
    // translation was None. This test locks in the added translation.
    #[test]
    fn should_translate_fufu_interjection() {
        ensure_dictionaries();
        let text = "ふふ、ストーブついてますし";
        let tokens = super::super::tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, text);

        let fufu = results
            .iter()
            .find(|t| t.surface_form == "ふふ")
            .expect("「ふふ」interjection token should exist");
        assert!(
            fufu.translation.is_some(),
            "「ふふ」interjection should have a translation, got: {:?}",
            fufu
        );
    }

    // --- Phrase 10: ついてますし (the し reason particle) ---
    // Before the fix, 〜し (reason/listing particle "besides / and what's more")
    // existed as a grammar rule, but that rule had neither keywords nor a
    // format_map, so it could never be matched by resolve_grammar_label.
    // Additionally 「し」 base_form resolved to the する-stem homonym, yielding the
    // verb meaning "to do" instead of the particle. This test locks in the
    // keyword activation + homonym blacklist that resolve it.
    #[test]
    fn should_label_shi_reason_particle_grammar() {
        ensure_dictionaries();
        let text = "ストーブついてますし、大丈夫ですよ";
        let tokens = super::super::tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, text);

        let shi = results
            .iter()
            .find(|t| t.surface_form == "し" && t.pos == PartOfSpeech::Particle)
            .expect("「し」particle token should exist");
        assert!(
            shi.grammar_label.is_some(),
            "〜し reason particle should carry a grammar_label, got translation={:?} grammar={:?}",
            shi.translation,
            shi.grammar_label
        );
    }

    // --- Cross-cutting false positive: plain topic は (phrases 1 & 2) ---
    // Before the fix, the grammar rule ～は～ほど～ない listed 「は」 itself as a
    // keyword, so EVERY topic particle は was mislabeled with that pattern even in
    // a plain ～は～です sentence. This test locks in the AND-semantics fix that
    // prevents the bare-keyword false positive.
    #[test]
    fn should_not_mislabel_plain_ha_particle_as_hahodo_nai() {
        ensure_dictionaries();
        let text = "私は学生です";
        let tokens = super::super::tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, text);

        let ha = results
            .iter()
            .find(|t| t.surface_form == "は" && t.pos == PartOfSpeech::Particle)
            .expect("「は」particle token should exist");
        let mislabeled = ha
            .grammar_label
            .as_deref()
            .is_some_and(|label| label.contains("ほど"));
        assert!(
            !mislabeled,
            "plain topic 「は」must not get the ～は～ほど～ない label, got: {:?}",
            ha.grammar_label
        );
    }

    // Characterization test: the input `言っイッてみただけ` contains an OCR/furigana
    // artifact (the katakana イッ inside a hiragana verb form). Lindera cannot
    // reconstruct the canonical te-form 言って here, but it must still recognize
    // the verb stem 言っ and surface its dictionary form 言う so the translation
    // pipeline can work. The katakana イッ being read as the numeral 一 is a known
    // tokenizer/dictionary limitation and is out of scope for translation.rs.
    #[test]
    fn should_still_extract_iu_base_form_from_mixed_kana_kanji_input() {
        ensure_dictionaries();
        let text = "言っイッてみただけ";
        let tokens = super::super::tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::English, text);

        let iu = results.iter().find(|t| t.base_form == "言う");
        assert!(
            iu.is_some(),
            "Should find 言う base_form in '{}', tokens: {:?}",
            text,
            results
                .iter()
                .map(|t| (&t.surface_form, &t.base_form))
                .collect::<Vec<_>>()
        );
    }

    // Particle 「し」 lists reason particle. Its base_form resolves to a homonym
    // (為る "to do" / する-stem), so without the homonym blacklist the particle
    // would be translated as the verb instead of being left untranslated.
    #[test]
    fn should_suppress_homonym_translation_for_particle_shi() {
        ensure_dictionaries();
        let text = "ストーブついてますし、大丈夫ですよ";
        let tokens = super::super::tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, text);

        let shi = results
            .iter()
            .find(|t| t.surface_form == "し" && t.pos == PartOfSpeech::Particle)
            .expect("Particle 「し」 token should exist (tokenization confirmed by repro test)");
        assert!(
            shi.translation.is_none(),
            "Particle 「し」must not be translated via homonym (為る=делать), got: {:?}",
            shi.translation
        );
    }

    // ～すぎる keyword rule must also resolve for adjective-stem compounds:
    // 複雑すぎる is tokenized by Lindera as 複雑 (NaAdjective) + すぎる, and the
    // auxiliary must surface the ～すぎる grammar_label even though format_map
    // would otherwise match the longer phrase.
    #[test]
    fn should_detect_sugiru_label_for_compound_word() {
        ensure_dictionaries();
        let text = "この問題は複雑すぎる";
        let tokens = super::super::tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, text);

        let has_sugiru_label = results.iter().any(|t| {
            t.grammar_label
                .as_deref()
                .is_some_and(|label| label.contains("すぎる"))
        });
        assert!(
            has_sugiru_label,
            "～すぎる should be detected for 複雑すぎる, labels: {:?}",
            results
                .iter()
                .filter(|t| t.grammar_label.is_some())
                .map(|t| (&t.surface_form, &t.grammar_label))
                .collect::<Vec<_>>()
        );
    }

    // ～すぎる detection for noun-stem compounds in te-form: 複雑すぎて is split
    // by Lindera into 複雑 (Noun) + すぎ (Verb, base 過ぎる) + て. The sugiru
    // rule's format_map covers only Verb and IAdjective (no Noun entry), so
    // format("複雑", Noun) errors before any matching can happen. The keyword
    // whitelist on the rule catches the auxiliary token すぎ. This test locks
    // in that noun-stem compounds in te-form are still labeled correctly via
    // the keyword fallback.
    #[test]
    fn should_detect_sugiru_te_form_for_noun_stem_compound() {
        ensure_dictionaries();
        let text = "この問題は複雑すぎて困っている";
        let tokens = super::super::tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, text);

        let has_sugiru = results.iter().any(|t| {
            t.grammar_label
                .as_deref()
                .is_some_and(|label| label.contains("すぎる"))
        });
        assert!(
            has_sugiru,
            "すぎる should be detected for 複雑すぎて, labels: {:?}",
            results
                .iter()
                .filter(|t| t.grammar_label.is_some())
                .map(|t| (&t.surface_form, &t.grammar_label))
                .collect::<Vec<_>>()
        );
    }

    // Standalone grammar markers (particle / noun / auxiliary) that previously
    // carried no grammar_label because their rule had only a format_map and no
    // keywords. Keyword matching fires on the token surface, so once a keyword
    // is added the marker token itself is labeled instead of staying bare.
    fn assert_marker_has_grammar_label(text: &str, marker_surface: &str, expected_label: &str) {
        ensure_dictionaries();
        let tokens = super::super::tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, text);
        let marker = results
            .iter()
            .find(|t| t.surface_form == marker_surface)
            .unwrap_or_else(|| panic!(
                "「{marker_surface}」 token should exist in '{text}', got surfaces: {surfaces:?}",
                surfaces = results.iter().map(|t| t.surface_form.as_str()).collect::<Vec<_>>()
            ));
        assert_eq!(
            marker.grammar_label.as_deref(),
            Some(expected_label),
            "「{marker_surface}」 in '{text}' grammar_label mismatch, token: {marker:?}",
        );
    }

    #[test]
    fn should_label_te_particle_via_keyword() {
        assert_marker_has_grammar_label("食べて飲んで", "て", "～て");
    }

    #[test]
    fn should_label_tara_conditional_via_keyword() {
        assert_marker_has_grammar_label("食べたら", "たら", "～たら");
    }

    #[test]
    fn should_label_okage_thanks_to_via_keyword() {
        assert_marker_has_grammar_label("彼のおかげで成功した", "おかげ", "～おかげで");
    }

    #[test]
    fn should_label_sei_because_of_via_keyword() {
        assert_marker_has_grammar_label("失敗は彼のせいだ", "せい", "～せいで");
    }

    #[test]
    fn should_label_totan_the_moment_via_keyword() {
        assert_marker_has_grammar_label(
            "ドアを開けたとたん猫が逃げた",
            "とたん",
            "～たとたん（に）",
        );
    }

    // そうだ after an i-adjective stem (美味し, base 美味しい) is 様態
    // (appearance): "looks delicious". Lindera surfaces the stem with
    // surface != base, which resolve_sou_da_label detects.
    #[test]
    fn should_label_sou_as_appearance_after_adjective_stem() {
        ensure_dictionaries();
        let text = "この料理は美味しそうだ";
        let tokens = super::super::tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, text);

        let sou = results
            .iter()
            .find(|t| t.surface_form == "そう")
            .expect("「そう」token should exist");
        assert_eq!(
            sou.grammar_label.as_deref(),
            Some("～そうだ（様態）"),
            "そう after adjective stem should be 様態 (appearance), got: {:?}",
            sou
        );
    }

    // そうだ after a verb dictionary form (来る, surface == base) is 伝聞
    // (hearsay): "I heard he will come".
    #[test]
    fn should_label_sou_as_hearsay_after_dictionary_form() {
        ensure_dictionaries();
        let text = "田中さんは来るそうだ";
        let tokens = super::super::tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, text);

        let sou = results
            .iter()
            .find(|t| t.surface_form == "そう")
            .expect("「そう」token should exist");
        assert_eq!(
            sou.grammar_label.as_deref(),
            Some("～そうだ（伝聞）"),
            "そう after dictionary form should be 伝聞 (hearsay), got: {:?}",
            sou
        );
    }

    // そうだ after a past auxiliary (降った → 降っ + た) is 伝聞 (hearsay):
    // "I heard it rained". The token directly before そう is the た auxiliary.
    #[test]
    fn should_label_sou_as_hearsay_after_past_form() {
        ensure_dictionaries();
        let text = "雨が降ったそうだ";
        let tokens = super::super::tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, text);

        let sou = results
            .iter()
            .find(|t| t.surface_form == "そう")
            .expect("「そう」token should exist");
        assert_eq!(
            sou.grammar_label.as_deref(),
            Some("～そうだ（伝聞）"),
            "そう after past auxiliary should be 伝聞 (hearsay), got: {:?}",
            sou
        );
    }

    // Compound lemma そういう ("such") shares the そう prefix but is a distinct
    // word. The context-aware path must not attach a そうだ grammar_label to it.
    #[test]
    fn should_not_label_souiu_compound_as_sou_da_grammar() {
        ensure_dictionaries();
        let text = "そういうこと";
        let tokens = super::super::tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, text);

        let souiu = results
            .iter()
            .find(|t| t.surface_form == "そういう")
            .expect("「そういう」token should exist");
        let mislabeled = souiu
            .grammar_label
            .as_deref()
            .is_some_and(|label| label.contains("そうだ"));
        assert!(
            !mislabeled,
            "「そういう」must not carry a そうだ grammar_label, got: {:?}",
            souiu.grammar_label
        );
    }

    // そうだ after a copula だ following a noun/na-adj (元気だそうだ) is 伝聞:
    // the token directly before そう is the だ copula. Locks in the central
    // adaptation — noun/na-adj hearsay is detected via the intervening copula.
    #[test]
    fn should_label_sou_as_hearsay_after_copula_da() {
        ensure_dictionaries();
        let text = "元気だそうだ";
        let tokens = super::super::tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, text);

        let sou = results
            .iter()
            .find(|t| t.surface_form == "そう")
            .expect("「そう」token should exist");
        assert_eq!(
            sou.grammar_label.as_deref(),
            Some("～そうだ（伝聞）"),
            "そう after copula だ should be 伝聞 (hearsay), got: {:?}",
            sou
        );
    }

    // そうだ after an i-adjective past (美味しかった → 美味しかっ + た) is 伝聞.
    #[test]
    fn should_label_sou_as_hearsay_after_iadjective_past() {
        ensure_dictionaries();
        let text = "美味しかったそうだ";
        let tokens = super::super::tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, text);

        let sou = results
            .iter()
            .find(|t| t.surface_form == "そう")
            .expect("「そう」token should exist");
        assert_eq!(
            sou.grammar_label.as_deref(),
            Some("～そうだ（伝聞）"),
            "そう after i-adjective past should be 伝聞 (hearsay), got: {:?}",
            sou
        );
    }

    // A noun directly before そう (元気そうだ) is token-ambiguous (the 様態 stem
    // looks identical to the dictionary form), so it resolves to the combined
    // rule rather than forcing hearsay.
    #[test]
    fn should_label_sou_as_combined_after_bare_noun() {
        ensure_dictionaries();
        let text = "元気そうだ";
        let tokens = super::super::tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, text);

        let sou = results
            .iter()
            .find(|t| t.surface_form == "そう")
            .expect("「そう」token should exist");
        assert_eq!(
            sou.grammar_label.as_deref(),
            Some("～そうだ（様態・伝聞）"),
            "そう after a bare noun should fall back to the combined rule, got: {:?}",
            sou
        );
    }

    // Demonstrative/manner そう after a particle (私はそう思う) is NOT a そうだ
    // construction — it must defer to resolve_grammar_label so the manner rule
    // こう・そう・ああ・どう matches. Guards against the Combined-fallback false
    // positive where そう follows a non-predicate token.
    #[test]
    fn should_label_demonstrative_sou_as_manner_not_sou_da() {
        ensure_dictionaries();
        let text = "私はそう思う";
        let tokens = super::super::tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, text);

        let sou = results
            .iter()
            .find(|t| t.surface_form == "そう")
            .expect("「そう」token should exist");
        assert!(
            sou.grammar_label
                .as_deref()
                .is_some_and(|label| label.contains("そう") && !label.contains("そうだ")),
            "demonstrative そう should match the manner rule, not a そうだ rule, got: {:?}",
            sou.grammar_label
        );
    }

    // A new sentence after 。 breaks the predecessor chain — そう in the second
    // clause (食べた。そうだ。) is agreement, not hearsay about the first clause.
    #[test]
    fn should_not_attach_sou_da_label_across_sentence_boundary() {
        ensure_dictionaries();
        let text = "食べた。そうだ。";
        let tokens = super::super::tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, text);

        let sou = results
            .iter()
            .find(|t| t.surface_form == "そう")
            .expect("「そう」token should exist");
        assert!(
            sou.grammar_label
                .as_deref()
                .is_none_or(|label| !label.contains("そうだ（伝聞）")),
            "そう after a sentence boundary must not be labeled hearsay, got: {:?}",
            sou.grammar_label
        );
    }

    #[test]
    fn should_label_kore_sore_are_demonstratives_via_keyword() {
        ensure_dictionaries();
        let text = "これは本です";
        let tokens = super::super::tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, text);
        let kore = results
            .iter()
            .find(|t| t.surface_form == "これ")
            .expect("「これ」token should exist");
        assert!(
            kore.grammar_label
                .as_deref()
                .is_some_and(|l| l.contains("これ")),
            "「これ」should carry the これ・それ・あれ grammar_label via keyword, got: {:?}",
            kore
        );
    }

    #[test]
    fn should_label_san_honorific_suffix_via_keyword() {
        ensure_dictionaries();
        let text = "田中さん";
        let tokens = super::super::tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, text);
        let san = results
            .iter()
            .find(|t| t.surface_form == "さん")
            .expect("「さん」suffix token should exist");
        assert!(
            san.grammar_label
                .as_deref()
                .is_some_and(|l| l.contains("さん")),
            "「さん」should carry the ～さん honorific grammar_label via keyword, got: {:?}",
            san
        );
    }

    #[test]
    fn should_label_itsu_question_word_via_keyword() {
        ensure_dictionaries();
        let text = "いつ行きますか";
        let tokens = super::super::tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, text);
        let itsu = results
            .iter()
            .find(|t| t.surface_form == "いつ")
            .expect("「いつ」token should exist");
        assert!(
            itsu.grammar_label
                .as_deref()
                .is_some_and(|l| l.contains("いつ")),
            "「いつ」should carry the いつ question-word grammar_label via keyword, got: {:?}",
            itsu
        );
    }
}
