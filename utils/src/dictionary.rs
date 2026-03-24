use flate2::read::DeflateDecoder;
use origa::domain::{DictionaryData, OrigaError, init_dictionary, is_dictionary_loaded};
use std::fs;
use std::io::Read;
use std::path::PathBuf;

/// Decompresses deflate-compressed data
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

/// Loads the UniDic dictionary if not already loaded
pub fn load_dictionary() -> Result<(), OrigaError> {
    if is_dictionary_loaded() {
        return Ok(());
    }

    let dict_dir = find_dictionary_directory()?;

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

    init_dictionary(data)
}

/// Finds the dictionary directory path
fn find_dictionary_directory() -> Result<PathBuf, OrigaError> {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let mut dict_dir = PathBuf::from(manifest_dir);

    if dict_dir.ends_with("tokenizer") {
        dict_dir.pop();
    }

    dict_dir = dict_dir
        .join("origa_ui")
        .join("public")
        .join("dictionaries")
        .join("unidic");

    if !dict_dir.exists() {
        dict_dir = PathBuf::from("origa_ui/public/dictionaries/unidic");
    }

    if !dict_dir.exists() {
        return Err(OrigaError::TokenizerError {
            reason: format!("Dictionary directory not found: {}", dict_dir.display()),
        });
    }

    Ok(dict_dir)
}
