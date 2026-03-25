use std::io::Read;
use flate2::read::DeflateDecoder;
use origa::domain::{DictionaryData, OrigaError, init_dictionary, init_dictionary_from_rkyv, is_dictionary_loaded};
use crate::repository::{get_cached_dictionary_rkyv, save_dictionary_to_cache_rkyv};
use crate::utils::yield_to_browser;
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
async fn fetch_file(
    window: &web_sys::Window,
    url: String,
    filename: &str,
    field: &str,
) -> Result<(String, Vec<u8>), OrigaError> {
    use leptos::wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;
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
    let decompressed = if field == "metadata" {
        bytes
    } else {
        decompress(bytes)?
    };
    Ok((field.to_string(), decompressed))
}
pub async fn load_dictionary() -> Result<(), OrigaError> {
    if is_dictionary_loaded() {
        tracing::debug!("📖 Dictionary already loaded");
        return Ok(());
    }
    let start = now_ms();
    tracing::info!("📖 Loading Unidic dictionary...");
    // Try to load from rkyv cache first
    match get_cached_dictionary_rkyv().await {
        Ok(Some(bytes)) => {
            tracing::info!("📖 Dictionary found in cache (rkyv), {} bytes", bytes.len());
            yield_to_browser().await;
            init_dictionary_from_rkyv(&bytes)?;
            tracing::info!(
                "📖 Dictionary loaded from rkyv cache ({:.2}s)",
                (now_ms() - start) / 1000.0
            );
            return Ok(());
        }
        Ok(None) => {
            tracing::debug!("📖 No rkyv cache found, loading from network");
        }
        Err(e) => {
            tracing::warn!("Cache read failed, loading from network: {:?}", e);
        }
    }
    let data = load_dictionary_from_network().await?;
    let data_clone = data.clone();
    yield_to_browser().await;
    init_dictionary(data)?;
    let cache_start = now_ms();
    if let Err(e) = save_dictionary_to_cache_rkyv(&data_clone).await {
        tracing::warn!("Failed to cache dictionary: {:?}", e);
    } else {
        tracing::info!(
            "📖 Dictionary cached (rkyv) ({:.2}s)",
            (now_ms() - cache_start) / 1000.0
        );
    }
    tracing::info!(
        "📖 Dictionary loaded from network ({:.2}s)",
        (now_ms() - start) / 1000.0
    );
    Ok(())
}
fn now_ms() -> f64 {
    web_sys::window()
        .and_then(|w| w.performance())
        .map(|p| p.now())
        .unwrap_or(0.0)
}
async fn load_dictionary_from_network() -> Result<DictionaryData, OrigaError> {
    use futures::future::join_all;
    tracing::info!("📖 Fetching dictionary files...");
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
    let fetch_start = now_ms();
    let fetch_futures: Vec<_> = files
        .iter()
        .map(|(field, filename)| {
            let url = crate::core::config::public_url(&format!(
                "/public/dictionaries/unidic/{}",
                filename
            ));
            fetch_file(&window, url, filename, field)
        })
        .collect();
    let results = join_all(fetch_futures).await;
    tracing::info!(
        "📖 Dictionary files fetched ({:.2}s)",
        (now_ms() - fetch_start) / 1000.0
    );
    let results: Vec<_> = results.into_iter().collect::<Result<Vec<_>, _>>()?;
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
    for (field, decompressed) in results {
        match field.as_str() {
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
    Ok(data)
}
