use flate2::read::DeflateDecoder;
use origa::domain::{DictionaryData, OrigaError, init_dictionary, is_dictionary_loaded};
use std::fs;
use std::io::Read;
use std::path::PathBuf;

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

const DICT_FILES: &[&str] = &[
    "char_def.bin",
    "matrix.mtx",
    "dict.da",
    "dict.vals",
    "unk.bin",
    "dict.wordsidx",
    "dict.words",
    "metadata.json",
];

const CDN_DICT_PATH: &str = "/public/dictionaries/unidic/cache/dictionary-data";

fn cache_dir() -> PathBuf {
    PathBuf::from("target/cdn-cache")
}

fn download_from_cdn(cache: &std::path::Path) -> Result<(), OrigaError> {
    let base_url =
        crate::signing::cdn_url(CDN_DICT_PATH).map_err(|e| OrigaError::TokenizerError {
            reason: format!("CDN signing failed: {e}"),
        })?;

    let client = reqwest::blocking::Client::new();
    for &name in DICT_FILES {
        let url = format!("{base_url}/{name}");
        let resp = client
            .get(&url)
            .send()
            .and_then(|r| r.error_for_status())
            .and_then(|r| r.bytes())
            .map_err(|e| OrigaError::TokenizerError {
                reason: format!("Failed to download {name} from CDN: {e}"),
            })?;
        fs::write(cache.join(name), &resp).map_err(|e| OrigaError::TokenizerError {
            reason: format!("Failed to write {name}: {e}"),
        })?;
    }

    Ok(())
}

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

    if dict_dir.exists() {
        return Ok(dict_dir);
    }

    dict_dir = PathBuf::from("origa_ui/public/dictionaries/unidic");
    if dict_dir.exists() {
        return Ok(dict_dir);
    }

    let cache = cache_dir();
    if cache.exists() {
        tracing::info!("using cached CDN dictionary from {}", cache.display());
        return Ok(cache);
    }

    tracing::info!("local dictionary not found, downloading from CDN...");
    fs::create_dir_all(&cache).map_err(|e| OrigaError::TokenizerError {
        reason: format!("Failed to create cache dir: {e}"),
    })?;

    download_from_cdn(&cache)?;
    tracing::info!("dictionary cached to {}", cache.display());

    Ok(cache)
}
