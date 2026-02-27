use std::io::Read;

use flate2::read::DeflateDecoder;
use origa::domain::{DictionaryData, OrigaError, init_dictionary, is_dictionary_loaded};

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

    use leptos::wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;

    let base_url = "/public/dictionaries/unidic/";
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

    let window = web_sys::window().ok_or_else(|| OrigaError::TokenizerError {
        reason: "No window found".to_string(),
    })?;
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
        let resp_value = JsFuture::from(window.fetch_with_str(&url))
            .await
            .map_err(|e| OrigaError::TokenizerError {
                reason: format!("Failed to fetch {}: {:?}", filename, e),
            })?;

        let resp: web_sys::Response =
            resp_value
                .dyn_into()
                .map_err(|e| OrigaError::TokenizerError {
                    reason: format!("Failed to cast response for {}: {:?}", filename, e),
                })?;

        if !resp.ok() {
            return Err(OrigaError::TokenizerError {
                reason: format!("Failed to fetch {}: HTTP {}", filename, resp.status()),
            });
        }

        let array_buffer_promise = resp
            .array_buffer()
            .map_err(|e| OrigaError::TokenizerError {
                reason: format!("Failed to get array buffer for {}: {:?}", filename, e),
            })?;

        let array_buffer_value =
            JsFuture::from(array_buffer_promise)
                .await
                .map_err(|e| OrigaError::TokenizerError {
                    reason: format!("Failed to await array buffer for {}: {:?}", filename, e),
                })?;

        let array_buffer = js_sys::ArrayBuffer::from(array_buffer_value);
        let bytes = js_sys::Uint8Array::new(&array_buffer).to_vec();

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

    init_dictionary(data)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn load_dictionary() -> Result<(), OrigaError> {
    if is_dictionary_loaded() {
        return Ok(());
    }

    use std::{env, fs, path::PathBuf};

    let dict_dir = if let Ok(out_dir) = env::var("OUT_DIR") {
        let out_dict = PathBuf::from(out_dir).join("lindera-unidic");
        if out_dict.exists() {
            out_dict
        } else {
            let manifest_dir =
                env::var("CARGO_MANIFEST_DIR").map_err(|_| OrigaError::TokenizerError {
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
            env::var("CARGO_MANIFEST_DIR").map_err(|_| OrigaError::TokenizerError {
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

    init_dictionary(data)
}
