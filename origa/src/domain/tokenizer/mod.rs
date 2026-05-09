mod part_of_speech;
pub mod translation;

use std::sync::OnceLock;

pub use part_of_speech::PartOfSpeech;
pub use translation::{TokenTranslation, lookup_tokens_translations};

use crate::domain::{JapaneseChar, OrigaError};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TokenInfo {
    orthographic_base_form: String,
    phonological_base_form: String,
    orthographic_surface_form: String,
    phonological_surface_form: String,
    part_of_speech: PartOfSpeech,
}

impl TokenInfo {
    pub fn orthographic_base_form(&self) -> &str {
        &self.orthographic_base_form
    }

    pub fn phonological_base_form(&self) -> &str {
        &self.phonological_base_form
    }

    pub fn orthographic_surface_form(&self) -> &str {
        &self.orthographic_surface_form
    }

    pub fn phonological_surface_form(&self) -> &str {
        &self.phonological_surface_form
    }

    pub fn part_of_speech(&self) -> &PartOfSpeech {
        &self.part_of_speech
    }
}

#[derive(
    Clone, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize,
)]
pub struct DictionaryData {
    pub char_def: Vec<u8>,
    pub matrix: Vec<u8>,
    pub dict_da: Vec<u8>,
    pub dict_vals: Vec<u8>,
    pub unk: Vec<u8>,
    pub words_idx: Vec<u8>,
    pub words: Vec<u8>,
    pub metadata: Vec<u8>,
}

/// Convert DictionaryData to rkyv bytes for storage
pub fn serialize_dictionary_to_rkyv(data: &DictionaryData) -> Result<Vec<u8>, OrigaError> {
    let bytes =
        rkyv::to_bytes::<rkyv::rancor::Error>(data).map_err(|e| OrigaError::TokenizerError {
            reason: format!("Failed to serialize dictionary: {}", e),
        })?;
    Ok(bytes.to_vec())
}

/// Initialize dictionary from rkyv bytes directly (zero-copy access)
pub fn init_dictionary_from_rkyv(bytes: &[u8]) -> Result<(), OrigaError> {
    let archived =
        rkyv::access::<ArchivedDictionaryData, rkyv::rancor::Error>(bytes).map_err(|e| {
            OrigaError::TokenizerError {
                reason: format!("Failed to validate dictionary data: {:?}", e),
            }
        })?;

    let data = DictionaryData {
        char_def: archived.char_def.to_vec(),
        matrix: archived.matrix.to_vec(),
        dict_da: archived.dict_da.to_vec(),
        dict_vals: archived.dict_vals.to_vec(),
        unk: archived.unk.to_vec(),
        words_idx: archived.words_idx.to_vec(),
        words: archived.words.to_vec(),
        metadata: archived.metadata.to_vec(),
    };

    init_dictionary(data)
}

static DICTIONARY_DATA: OnceLock<DictionaryData> = OnceLock::new();
static TOKENIZER: OnceLock<lindera::tokenizer::Tokenizer> = OnceLock::new();

pub fn is_dictionary_loaded() -> bool {
    TOKENIZER.get().is_some()
}

pub fn init_dictionary(data: DictionaryData) -> Result<(), OrigaError> {
    let _ = DICTIONARY_DATA.get_or_init(|| data);
    init_tokenizer()
}

fn init_tokenizer() -> Result<(), OrigaError> {
    let data = DICTIONARY_DATA.get().ok_or(OrigaError::TokenizerError {
        reason: "Dictionary data not loaded".to_string(),
    })?;

    let metadata = lindera_dictionary::dictionary::metadata::Metadata::load(&data.metadata)
        .map_err(|e| OrigaError::TokenizerError {
            reason: format!("Failed to load metadata: {}", e),
        })?;

    let prefix_dictionary =
        lindera_dictionary::dictionary::prefix_dictionary::PrefixDictionary::load(
            data.dict_da.clone(),
            data.dict_vals.clone(),
            data.words_idx.clone(),
            data.words.clone(),
            true,
        );

    let connection_cost_matrix =
        lindera_dictionary::dictionary::connection_cost_matrix::ConnectionCostMatrix::load(
            data.matrix.clone(),
        );

    let character_definition =
        lindera_dictionary::dictionary::character_definition::CharacterDefinition::load(
            &data.char_def,
        )
        .map_err(|e| OrigaError::TokenizerError {
            reason: format!("Failed to load character definition: {}", e),
        })?;

    let unknown_dictionary =
        lindera_dictionary::dictionary::unknown_dictionary::UnknownDictionary::load(&data.unk)
            .map_err(|e| OrigaError::TokenizerError {
                reason: format!("Failed to load unknown dictionary: {}", e),
            })?;

    let dictionary = lindera_dictionary::dictionary::Dictionary {
        prefix_dictionary,
        connection_cost_matrix,
        character_definition,
        unknown_dictionary,
        metadata,
    };

    let segmenter =
        lindera::segmenter::Segmenter::new(lindera::mode::Mode::Normal, dictionary, None);

    let tokenizer = lindera::tokenizer::Tokenizer::new(segmenter);

    let _ = TOKENIZER.get_or_init(|| tokenizer);
    Ok(())
}

pub fn tokenize_text(text: &str) -> Result<Vec<TokenInfo>, OrigaError> {
    let mut result = Vec::new();
    let mut current_segment = String::new();
    let mut is_current_japanese = false;

    for ch in text.chars() {
        let is_japanese = ch.is_japanese();

        if current_segment.is_empty() {
            is_current_japanese = is_japanese;
            current_segment.push(ch);
        } else if is_japanese == is_current_japanese {
            current_segment.push(ch);
        } else {
            flush_segment(&current_segment, is_current_japanese, &mut result)?;
            current_segment = ch.to_string();
            is_current_japanese = is_japanese;
        }
    }

    if !current_segment.is_empty() {
        flush_segment(&current_segment, is_current_japanese, &mut result)?;
    }

    Ok(result)
}

fn flush_segment(
    segment: &str,
    is_japanese: bool,
    result: &mut Vec<TokenInfo>,
) -> Result<(), OrigaError> {
    if is_japanese {
        let tokenizer = TOKENIZER.get().ok_or(OrigaError::TokenizerError {
            reason: "Dictionary not loaded. Call init_dictionary() first.".to_string(),
        })?;
        let mut tokens = tokenizer
            .tokenize(segment)
            .map_err(|e| OrigaError::TokenizerError {
                reason: e.to_string(),
            })?;
        for token in tokens.iter_mut() {
            result.push(token_to_token_info(token));
        }
    } else {
        let is_whitespace = segment.chars().all(|c| c.is_whitespace());
        result.push(non_japanese_token(segment, is_whitespace));
    }
    Ok(())
}

fn token_to_token_info(token: &mut lindera::token::Token) -> TokenInfo {
    use crate::domain::JapaneseText;

    let lexeme_raw = token.get("lexeme").unwrap_or_default().to_string();
    let lexeme_stripped: &str = if let Some((japanese, _english)) = lexeme_raw.split_once('-') {
        japanese
    } else {
        &lexeme_raw
    };

    let orth_base = token
        .get("orthographic_base_form")
        .unwrap_or_default()
        .to_string();

    let orthographic_base_form = if lexeme_stripped.contains_kanji() && !orth_base.contains_kanji()
    {
        lexeme_stripped.to_string()
    } else {
        orth_base.to_string()
    };

    let mut part_of_speech: PartOfSpeech = token
        .get("part_of_speech")
        .unwrap_or_default()
        .parse()
        .unwrap_or(PartOfSpeech::Unspecified);

    if part_of_speech == PartOfSpeech::Noun {
        let pos_subcategory = token
            .get("part_of_speech_subcategory_1")
            .unwrap_or_default();
        if pos_subcategory == "固有名詞" {
            part_of_speech = PartOfSpeech::ProperNoun;
        }
    }

    if orthographic_base_form == "*"
        || orthographic_base_form == "×"
        || orthographic_base_form == "•"
    {
        part_of_speech = PartOfSpeech::Symbol;
    }

    TokenInfo {
        orthographic_base_form,
        phonological_base_form: token
            .get("phonological_base_form")
            .unwrap_or_default()
            .to_string(),
        orthographic_surface_form: token
            .get("orthographic_surface_form")
            .unwrap_or_default()
            .to_string(),
        phonological_surface_form: token
            .get("phonological_surface_form")
            .unwrap_or_default()
            .to_string(),
        part_of_speech,
    }
}

fn non_japanese_token(segment: &str, is_whitespace: bool) -> TokenInfo {
    let pos = if is_whitespace {
        PartOfSpeech::Whitespace
    } else {
        PartOfSpeech::Symbol
    };
    TokenInfo {
        orthographic_base_form: segment.to_string(),
        phonological_base_form: segment.to_string(),
        orthographic_surface_form: segment.to_string(),
        phonological_surface_form: segment.to_string(),
        part_of_speech: pos,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ensure_dictionary() {
        if !is_dictionary_loaded() {
            let data = create_test_dictionary_data();
            let _ = init_dictionary(data);
        }
    }

    fn create_test_dictionary_data() -> DictionaryData {
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

        DictionaryData {
            char_def: decompress(read_file("char_def.bin")),
            matrix: decompress(read_file("matrix.mtx")),
            dict_da: decompress(read_file("dict.da")),
            dict_vals: decompress(read_file("dict.vals")),
            unk: decompress(read_file("unk.bin")),
            words_idx: decompress(read_file("dict.wordsidx")),
            words: decompress(read_file("dict.words")),
            metadata: read_file("metadata.json"),
        }
    }

    #[test]
    fn should_return_base_form_for_verb() {
        ensure_dictionary();
        let tokens = tokenize_text("食べます").unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].orthographic_base_form, "食べる");
        assert_eq!(tokens[0].phonological_base_form, "タベル");
    }

    #[test]
    fn should_return_base_form_for_noun() {
        ensure_dictionary();
        let tokens = tokenize_text("食べ物").unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].orthographic_base_form, "食べ物");
        assert_eq!(tokens[0].phonological_base_form, "タベモノ");
    }

    #[test]
    fn should_return_base_form_for_adjective() {
        ensure_dictionary();
        let tokens = tokenize_text("美味しい").unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].orthographic_base_form, "美味しい");
        assert_eq!(tokens[0].phonological_base_form, "オイシー");
    }

    #[test]
    fn should_return_base_form_for_hiragana() {
        ensure_dictionary();
        let tokens = tokenize_text("たべます").unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].orthographic_base_form, "食べる");
        assert_eq!(tokens[0].phonological_base_form, "タベル");
    }

    #[test]
    fn should_return_base_form_for_hiragana_2() {
        ensure_dictionary();
        let tokens = tokenize_text("こんや").unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].orthographic_base_form, "今夜");
        assert_eq!(tokens[0].phonological_base_form, "コンヤ");
    }

    #[test]
    fn should_return_surface_form_for_verb() {
        ensure_dictionary();
        let tokens = tokenize_text("食べます").unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].orthographic_surface_form, "食べ");
        assert_eq!(tokens[0].phonological_surface_form, "タベ");
    }

    #[test]
    fn should_return_surface_form_for_noun() {
        ensure_dictionary();
        let tokens = tokenize_text("食べ物").unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].orthographic_surface_form, "食べ物");
        assert_eq!(tokens[0].phonological_surface_form, "タベモノ");
    }

    #[test]
    fn should_return_surface_form_for_adjective() {
        ensure_dictionary();
        let tokens = tokenize_text("美味しい").unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].orthographic_surface_form, "美味しい");
        assert_eq!(tokens[0].phonological_surface_form, "オイシー");
    }

    #[test]
    fn should_return_surface_form_for_hiragana() {
        ensure_dictionary();
        let tokens = tokenize_text("たべます").unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].orthographic_surface_form, "たべ");
        assert_eq!(tokens[0].phonological_surface_form, "タベ");
    }

    #[test]
    fn should_clean_katakana_base_form() {
        ensure_dictionary();
        let tokens = tokenize_text("アニメ").unwrap();
        assert_eq!(tokens.len(), 1);
        // UniDic typically returns "アニメ-animation" for lexeme
        // Our modification should have stripped "-animation"
        assert_eq!(tokens[0].orthographic_base_form, "アニメ");
    }

    #[test]
    fn should_ignore_junk_symbols_as_vocabulary() {
        ensure_dictionary();
        // Symbols like * and x should be forced to Symbol POS
        let tokens = tokenize_text("アーニャ*ダミアン×ヨル").unwrap();
        for token in tokens {
            let base = token.orthographic_base_form();
            if base == "*" || base == "×" {
                assert!(
                    !token.part_of_speech().is_vocabulary_word(),
                    "Symbol '{}' (POS {:?}) should not be a vocabulary word",
                    base,
                    token.part_of_speech()
                );
                assert_eq!(token.part_of_speech(), &PartOfSpeech::Symbol);
            }
        }
    }

    #[test]
    fn should_preserve_kanji_for_proper_nouns() {
        ensure_dictionary();
        let tokens = tokenize_text("名古屋 横浜").unwrap();
        assert_eq!(tokens[0].orthographic_base_form, "名古屋");
        assert_eq!(tokens[1].part_of_speech(), &PartOfSpeech::Whitespace);
        assert_eq!(tokens[2].orthographic_base_form, "横浜");
    }

    #[test]
    fn should_detect_proper_noun() {
        ensure_dictionary();
        let tokens = tokenize_text("名古屋").unwrap();
        assert_eq!(tokens[0].part_of_speech(), &PartOfSpeech::ProperNoun);
        assert!(tokens[0].part_of_speech().is_vocabulary_word());
    }

    #[test]
    fn should_detect_common_noun_as_noun_not_proper() {
        ensure_dictionary();
        let tokens = tokenize_text("食べ物").unwrap();
        assert_eq!(tokens[0].part_of_speech(), &PartOfSpeech::Noun);
    }

    #[test]
    fn should_preserve_digits_in_tokenization() {
        ensure_dictionary();
        let tokens = tokenize_text("第3課").unwrap();
        let digit_token = tokens.iter().find(|t| t.orthographic_surface_form() == "3");
        assert!(
            digit_token.is_some(),
            "Expected digit token '3', got: {:?}",
            tokens
                .iter()
                .map(|t| t.orthographic_surface_form())
                .collect::<Vec<_>>()
        );
        assert_eq!(digit_token.unwrap().part_of_speech(), &PartOfSpeech::Symbol);
    }

    #[test]
    fn should_preserve_non_japanese_as_symbol() {
        ensure_dictionary();
        let tokens = tokenize_text("hello食べ物world").unwrap();
        let hello_token = tokens
            .iter()
            .find(|t| t.orthographic_surface_form() == "hello");
        assert!(hello_token.is_some());
        assert_eq!(hello_token.unwrap().part_of_speech(), &PartOfSpeech::Symbol);
        let world_token = tokens
            .iter()
            .find(|t| t.orthographic_surface_form() == "world");
        assert!(world_token.is_some());
        assert_eq!(world_token.unwrap().part_of_speech(), &PartOfSpeech::Symbol);
    }

    #[test]
    fn should_handle_only_non_japanese() {
        ensure_dictionary();
        let tokens = tokenize_text("123").unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].orthographic_surface_form(), "123");
        assert_eq!(tokens[0].part_of_speech(), &PartOfSpeech::Symbol);
    }

    #[test]
    fn should_handle_empty_text() {
        ensure_dictionary();
        let tokens = tokenize_text("").unwrap();
        assert!(tokens.is_empty());
    }

    #[test]
    fn should_handle_whitespace_as_whitespace_pos() {
        ensure_dictionary();
        let tokens = tokenize_text("   ").unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].part_of_speech(), &PartOfSpeech::Whitespace);
    }
}
