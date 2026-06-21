use serde::Serialize;

use super::{PartOfSpeech, TokenInfo};
use crate::dictionary::grammar::GRAMMAR_RULES;
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

pub fn lookup_tokens_translations(
    tokens: &[TokenInfo],
    native_language: &NativeLanguage,
    original_text: &str,
) -> Vec<TokenTranslation> {
    tokens
        .iter()
        .map(|token| {
            let base_form = token.orthographic_base_form().to_string();
            let translation = get_translation(&base_form, native_language);
            let grammar_label = resolve_grammar_label(token, native_language, original_text);

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
    if !is_vocab {
        for rule in rules.iter() {
            for group in rule.keywords().iter() {
                if group.iter().any(|kw| surface == kw) {
                    return Some(rule.content(native_language).title().to_string());
                }
            }
        }
    }

    // FormatAction detection for vocabulary tokens where surface != base (conjugated forms).
    // When multiple rules match, the most specific (longest formatted form) is preferred.
    if is_vocab && surface != base {
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
}
