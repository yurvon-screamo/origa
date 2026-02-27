use std::io::Read;

use crate::domain::is_dictionary_loaded;
use flate2::read::DeflateDecoder;

use crate::domain::OrigaError;
use crate::domain::tokenizer::{DICTIONARY_DATA, DictionaryData, TOKENIZER};

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
            let manifest_dir =
                std::env::var("CARGO_MANIFEST_DIR").map_err(|_| OrigaError::TokenizerError {
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
        let manifest_dir =
            std::env::var("CARGO_MANIFEST_DIR").map_err(|_| OrigaError::TokenizerError {
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
