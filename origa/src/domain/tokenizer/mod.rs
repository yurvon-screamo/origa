mod part_of_speech;

use std::io::Read;
use std::sync::OnceLock;

use flate2::read::DeflateDecoder;
pub use part_of_speech::PartOfSpeech;

use crate::domain::OrigaError;

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

struct DictionaryData {
    char_def: Vec<u8>,
    matrix: Vec<u8>,
    dict_da: Vec<u8>,
    dict_vals: Vec<u8>,
    unk: Vec<u8>,
    words_idx: Vec<u8>,
    words: Vec<u8>,
    metadata: Vec<u8>,
}

static DICTIONARY_DATA: OnceLock<DictionaryData> = OnceLock::new();
static TOKENIZER: OnceLock<lindera::tokenizer::Tokenizer> = OnceLock::new();

pub fn is_dictionary_loaded() -> bool {
    TOKENIZER.get().is_some()
}

fn decompress(data: Vec<u8>) -> Result<Vec<u8>, OrigaError> {
    let mut decoder = DeflateDecoder::new(&data[..]);
    let mut decompressed = Vec::new();
    decoder
        .read_to_end(&mut decompressed)
        .map_err(|e| OrigaError::TokenizerError {
            reason: format!("Decompression failed: {}", e),
        })?;
    Ok(decompressed)
}

#[cfg(target_arch = "wasm32")]
pub async fn load_dictionary() -> Result<(), OrigaError> {
    if is_dictionary_loaded() {
        return Ok(());
    }

    use reqwest::Client;

    let base_url = "/dictionaries/unidic/";
    let files = [
        ("char_def", "char_def.bin"),
        ("matrix", "matrix.mtx"),
        ("dict_da", "dict.da"),
        ("dict_vals", "dict.vals"),
        ("unk", "unk.bin"),
        ("words_idx", "dict.wordsidx"),
        ("words", "dict.words"),
        ("metadata", "metadata.json"),
    ];

    let client = Client::new();
    let mut data = DictionaryData {
        char_def: Vec::new(),
        matrix: Vec::new(),
        dict_da: Vec::new(),
        dict_vals: Vec::new(),
        unk: Vec::new(),
        words_idx: Vec::new(),
        words: Vec::new(),
        metadata: Vec::new(),
    };

    for (field, filename) in &files {
        let url = format!("{}{}", base_url, filename);
        let response = client
            .get(&url)
            .send()
            .await
            .map_err(|e| OrigaError::TokenizerError {
                reason: format!("Failed to fetch {}: {}", filename, e),
            })?;

        if !response.status().is_success() {
            return Err(OrigaError::TokenizerError {
                reason: format!("Failed to fetch {}: HTTP {}", filename, response.status()),
            });
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| OrigaError::TokenizerError {
                reason: format!("Failed to read {}: {}", filename, e),
            })?
            .to_vec();

        let decompressed = if *field == "metadata" {
            bytes
        } else {
            decompress(bytes)?
        };

        match *field {
            "char_def" => data.char_def = decompressed,
            "matrix" => data.matrix = decompressed,
            "dict_da" => data.dict_da = decompressed,
            "dict_vals" => data.dict_vals = decompressed,
            "unk" => data.unk = decompressed,
            "words_idx" => data.words_idx = decompressed,
            "words" => data.words = decompressed,
            "metadata" => data.metadata = decompressed,
            _ => {}
        }
    }

    let _ = DICTIONARY_DATA.get_or_init(|| data);
    init_tokenizer()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn load_dictionary() -> Result<(), OrigaError> {
    if is_dictionary_loaded() {
        return Ok(());
    }

    use std::fs;
    use std::path::PathBuf;

    let dict_dir = if let Ok(out_dir) = std::env::var("OUT_DIR") {
        let out_dict = PathBuf::from(out_dir).join("lindera-unidic");
        if out_dict.exists() {
            out_dict
        } else {
            let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
                .map_err(|_| OrigaError::TokenizerError {
                    reason: "CARGO_MANIFEST_DIR not set".to_string(),
                })?;
            PathBuf::from(manifest_dir)
                .parent()
                .ok_or_else(|| OrigaError::TokenizerError {
                    reason: "Cannot find parent directory".to_string(),
                })?
                .join("origa_ui")
                .join("public")
                .join("dictionaries")
                .join("unidic")
        }
    } else {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
            .map_err(|_| OrigaError::TokenizerError {
                reason: "CARGO_MANIFEST_DIR not set".to_string(),
            })?;
        PathBuf::from(manifest_dir)
            .parent()
            .ok_or_else(|| OrigaError::TokenizerError {
                reason: "Cannot find parent directory".to_string(),
            })?
            .join("origa_ui")
            .join("public")
            .join("dictionaries")
            .join("unidic")
    };

    if !dict_dir.exists() {
        return Err(OrigaError::TokenizerError {
            reason: format!("Dictionary directory not found: {}", dict_dir.display()),
        });
    }

    let read_file = |name: &str| -> Result<Vec<u8>, OrigaError> {
        fs::read(dict_dir.join(name)).map_err(|e| OrigaError::TokenizerError {
            reason: format!("Failed to read {}: {}", name, e),
        })
    };

    let data = DictionaryData {
        char_def: decompress(read_file("char_def.bin")?)?,
        matrix: decompress(read_file("matrix.mtx")?)?,
        dict_da: decompress(read_file("dict.da")?)?,
        dict_vals: decompress(read_file("dict.vals")?)?,
        unk: decompress(read_file("unk.bin")?)?,
        words_idx: decompress(read_file("dict.wordsidx")?)?,
        words: decompress(read_file("dict.words")?)?,
        metadata: read_file("metadata.json")?,
    };

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

    let prefix_dictionary = lindera_dictionary::dictionary::prefix_dictionary::PrefixDictionary::load(
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
    let tokenizer = TOKENIZER.get().ok_or(OrigaError::TokenizerError {
        reason: "Dictionary not loaded. Call load_dictionary() first.".to_string(),
    })?;

    let filtered_text = crate::domain::filter_japanese_text(text);
    let mut tokens =
        tokenizer
            .tokenize(&filtered_text)
            .map_err(|e| OrigaError::TokenizerError {
                reason: e.to_string(),
            })?;

    let token_infos = tokens
        .iter_mut()
        .map(|token| TokenInfo {
            orthographic_base_form: token.get("lexeme").unwrap_or_default().to_string(),
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
            part_of_speech: token
                .get("part_of_speech")
                .unwrap_or_default()
                .parse()
                .unwrap_or(PartOfSpeech::Unspecified),
        })
        .collect();

    Ok(token_infos)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ensure_dictionary() {
        if !is_dictionary_loaded() {
            let _ = load_dictionary();
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
}
