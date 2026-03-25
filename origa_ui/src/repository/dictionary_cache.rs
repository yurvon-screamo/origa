use serde::Serialize;
use serde::de::DeserializeOwned;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;

use origa::dictionary::grammar::GrammarData;
use origa::dictionary::kanji::KanjiData;
use origa::dictionary::radical::RadicalData;
use origa::dictionary::vocabulary::VocabularyChunkData;
use origa::domain::{DictionaryData, OrigaError};

use crate::core::config::urls;

const CACHE_NAME: &str = "origa-dictionary-rkyv-v2";

async fn open_cache() -> Result<web_sys::Cache, OrigaError> {
    let window = web_sys::window().ok_or_else(|| OrigaError::RepositoryError {
        reason: "No window found".to_string(),
    })?;

    let caches = js_sys::Reflect::get(&window, &wasm_bindgen::JsValue::from_str("caches"))
        .map_err(|e| OrigaError::RepositoryError {
            reason: format!("Cache API not available: {:?}", e),
        })?;

    let caches: web_sys::CacheStorage =
        caches.dyn_into().map_err(|e| OrigaError::RepositoryError {
            reason: format!("Failed to cast CacheStorage: {:?}", e),
        })?;

    let cache_promise = caches.open(CACHE_NAME);
    let cache: web_sys::Cache = JsFuture::from(cache_promise)
        .await
        .map_err(|e| OrigaError::RepositoryError {
            reason: format!("Failed to open cache: {:?}", e),
        })?
        .into();

    Ok(cache)
}

pub async fn get_cached_data<T: DeserializeOwned>(url: &str) -> Result<Option<T>, OrigaError> {
    let cache = open_cache().await?;

    let match_promise = cache.match_with_str(url);
    let response_option =
        JsFuture::from(match_promise)
            .await
            .map_err(|e| OrigaError::RepositoryError {
                reason: format!("Failed to check cache: {:?}", e),
            })?;

    if response_option.is_undefined() || response_option.is_null() {
        return Ok(None);
    }

    let response: web_sys::Response =
        response_option
            .dyn_into()
            .map_err(|e| OrigaError::RepositoryError {
                reason: format!("Failed to cast Response: {:?}", e),
            })?;

    if !response.ok() {
        return Ok(None);
    }

    let array_buffer_promise =
        response
            .array_buffer()
            .map_err(|e| OrigaError::RepositoryError {
                reason: format!("Failed to get array buffer: {:?}", e),
            })?;

    let array_buffer =
        JsFuture::from(array_buffer_promise)
            .await
            .map_err(|e| OrigaError::RepositoryError {
                reason: format!("Failed to read array buffer: {:?}", e),
            })?;

    let uint8_array = js_sys::Uint8Array::new(&array_buffer);
    let bytes = uint8_array.to_vec();

    let data: T = bincode::deserialize(&bytes).map_err(|e| OrigaError::RepositoryError {
        reason: format!("Failed to deserialize data: {:?}", e),
    })?;

    tracing::info!("Data loaded from Cache API: {}", url);
    Ok(Some(data))
}

pub async fn save_data_to_cache<T: Serialize>(url: &str, data: &T) -> Result<(), OrigaError> {
    let cache = open_cache().await?;

    let bytes = bincode::serialize(data).map_err(|e| OrigaError::RepositoryError {
        reason: format!("Failed to serialize data: {:?}", e),
    })?;

    let array_buffer = js_sys::ArrayBuffer::new(bytes.len() as u32);
    let view = js_sys::Uint8Array::new(&array_buffer);
    view.copy_from(&bytes);

    let response_init = web_sys::ResponseInit::new();
    response_init.set_status(200);
    response_init.set_status_text("OK");

    let blob_property_bag = web_sys::BlobPropertyBag::new();
    blob_property_bag.set_type("application/octet-stream");

    let blob_parts = js_sys::Array::new();
    blob_parts.push(&array_buffer);

    let blob =
        web_sys::Blob::new_with_buffer_source_sequence_and_options(&blob_parts, &blob_property_bag)
            .map_err(|e| OrigaError::RepositoryError {
                reason: format!("Failed to create blob: {:?}", e),
            })?;

    let response = web_sys::Response::new_with_opt_blob_and_init(Some(&blob), &response_init)
        .map_err(|e| OrigaError::RepositoryError {
            reason: format!("Failed to create response: {:?}", e),
        })?;

    let put_promise = cache.put_with_str(url, &response);
    JsFuture::from(put_promise)
        .await
        .map_err(|e| OrigaError::RepositoryError {
            reason: format!("Failed to save to cache: {:?}", e),
        })?;

    tracing::info!("Data saved to Cache API: {}", url);
    Ok(())
}

pub async fn get_cached_dictionary() -> Result<Option<DictionaryData>, OrigaError> {
    get_cached_data(urls().dictionary).await
}

pub async fn save_dictionary_to_cache(data: &DictionaryData) -> Result<(), OrigaError> {
    save_data_to_cache(urls().dictionary, data).await
}

pub async fn get_cached_vocabulary() -> Result<Option<VocabularyChunkData>, OrigaError> {
    get_cached_data(urls().vocabulary).await
}

pub async fn save_vocabulary_to_cache(data: &VocabularyChunkData) -> Result<(), OrigaError> {
    save_data_to_cache(urls().vocabulary, data).await
}

pub async fn get_cached_kanji() -> Result<Option<KanjiData>, OrigaError> {
    get_cached_data(urls().kanji).await
}

pub async fn save_kanji_to_cache(data: &KanjiData) -> Result<(), OrigaError> {
    save_data_to_cache(urls().kanji, data).await
}

pub async fn get_cached_radical() -> Result<Option<RadicalData>, OrigaError> {
    get_cached_data(urls().radical).await
}

pub async fn save_radical_to_cache(data: &RadicalData) -> Result<(), OrigaError> {
    save_data_to_cache(urls().radical, data).await
}

pub async fn get_cached_grammar() -> Result<Option<GrammarData>, OrigaError> {
    get_cached_data(urls().grammar).await
}

pub async fn save_grammar_to_cache(data: &GrammarData) -> Result<(), OrigaError> {
    save_data_to_cache(urls().grammar, data).await
}

// ============================================
// rkyv-based cache functions for vocabulary
// ============================================

/// Get cached vocabulary database as raw rkyv bytes
pub async fn get_cached_vocabulary_rkyv() -> Result<Option<Vec<u8>>, OrigaError> {
    get_cached_bytes(urls().vocabulary).await
}

/// Save vocabulary database to cache as rkyv bytes
pub async fn save_vocabulary_to_cache_rkyv(bytes: &[u8]) -> Result<(), OrigaError> {
    save_cached_bytes(urls().vocabulary, bytes).await
}

// ============================================
// rkyv-based cache functions for kanji
// ============================================

/// Get cached kanji as raw rkyv bytes
pub async fn get_cached_kanji_rkyv() -> Result<Option<Vec<u8>>, OrigaError> {
    get_cached_bytes(urls().kanji).await
}

/// Save kanji to cache as rkyv bytes
pub async fn save_kanji_to_cache_rkyv(bytes: &[u8]) -> Result<(), OrigaError> {
    save_cached_bytes(urls().kanji, bytes).await
}

// ============================================
// rkyv-based cache functions for radical
// ============================================

/// Get cached radical as raw rkyv bytes
pub async fn get_cached_radical_rkyv() -> Result<Option<Vec<u8>>, OrigaError> {
    get_cached_bytes(urls().radical).await
}

/// Save radical to cache as rkyv bytes
pub async fn save_radical_to_cache_rkyv(bytes: &[u8]) -> Result<(), OrigaError> {
    save_cached_bytes(urls().radical, bytes).await
}

// ============================================
// rkyv-based cache functions for grammar
// ============================================

/// Get cached grammar as raw rkyv bytes
pub async fn get_cached_grammar_rkyv() -> Result<Option<Vec<u8>>, OrigaError> {
    get_cached_bytes(urls().grammar).await
}

/// Save grammar to cache as rkyv bytes
pub async fn save_grammar_to_cache_rkyv(bytes: &[u8]) -> Result<(), OrigaError> {
    save_cached_bytes(urls().grammar, bytes).await
}

// ============================================
// Helper functions for raw bytes caching
// ============================================

async fn get_cached_bytes(url: &str) -> Result<Option<Vec<u8>>, OrigaError> {
    let start = now_ms();
    let cache = open_cache().await?;
    tracing::debug!("Cache opened ({:.2}s)", (now_ms() - start) / 1000.0);

    let match_start = now_ms();
    let match_promise = cache.match_with_str(url);
    let response_option = JsFuture::from(match_promise)
        .await
        .map_err(|e| OrigaError::RepositoryError {
            reason: format!("Failed to check cache: {:?}", e),
        })?;
    tracing::debug!(
        "Cache match checked ({:.2}s)",
        (now_ms() - match_start) / 1000.0
    );

    if response_option.is_undefined() || response_option.is_null() {
        return Ok(None);
    }

    let response: web_sys::Response = response_option
        .dyn_into()
        .map_err(|e| OrigaError::RepositoryError {
            reason: format!("Failed to cast Response: {:?}", e),
        })?;

    if !response.ok() {
        return Ok(None);
    }

    let array_buffer_promise = response
        .array_buffer()
        .map_err(|e| OrigaError::RepositoryError {
            reason: format!("Failed to get array buffer: {:?}", e),
        })?;

    let array_buffer = JsFuture::from(array_buffer_promise)
        .await
        .map_err(|e| OrigaError::RepositoryError {
            reason: format!("Failed to read array buffer: {:?}", e),
        })?;

    let uint8_array = js_sys::Uint8Array::new(&array_buffer);
    let bytes = uint8_array.to_vec();
    tracing::info!(
        "Cache loaded (rkyv): {} ({} bytes, {:.2}s)",
        url,
        bytes.len(),
        (now_ms() - start) / 1000.0
    );
    Ok(Some(bytes))
}

async fn save_cached_bytes(url: &str, bytes: &[u8]) -> Result<(), OrigaError> {
    let start = now_ms();
    let cache = open_cache().await?;
    tracing::debug!("Cache opened for save ({:.2}s)", (now_ms() - start) / 1000.0);

    let array_buffer = js_sys::ArrayBuffer::new(bytes.len() as u32);
    let view = js_sys::Uint8Array::new(&array_buffer);
    view.copy_from(bytes);

    let response_init = web_sys::ResponseInit::new();
    response_init.set_status(200);
    response_init.set_status_text("OK");

    let blob_property_bag = web_sys::BlobPropertyBag::new();
    blob_property_bag.set_type("application/octet-stream");

    let blob_parts = js_sys::Array::new();
    blob_parts.push(&array_buffer);

    let blob = web_sys::Blob::new_with_buffer_source_sequence_and_options(&blob_parts, &blob_property_bag)
        .map_err(|e| OrigaError::RepositoryError {
            reason: format!("Failed to create blob: {:?}", e),
        })?;

    let response = web_sys::Response::new_with_opt_blob_and_init(Some(&blob), &response_init)
        .map_err(|e| OrigaError::RepositoryError {
            reason: format!("Failed to create response: {:?}", e),
        })?;

    let put_start = now_ms();
    let put_promise = cache.put_with_str(url, &response);
    JsFuture::from(put_promise)
        .await
        .map_err(|e| OrigaError::RepositoryError {
            reason: format!("Failed to save to cache: {:?}", e),
        })?;
    tracing::info!(
        "Cache saved (rkyv): {} ({} bytes, put {:.2}s, total {:.2}s)",
        url,
        bytes.len(),
        (now_ms() - put_start) / 1000.0,
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

/// Get cached dictionary as raw rkyv bytes
pub async fn get_cached_dictionary_rkyv() -> Result<Option<Vec<u8>>, OrigaError> {
    let start = now_ms();
    let cache = open_cache().await?;
    tracing::debug!("Cache opened ({:.2}s)", (now_ms() - start) / 1000.0);

    let url = urls().dictionary;

    let match_start = now_ms();
    let match_promise = cache.match_with_str(url);
    let response_option = JsFuture::from(match_promise)
        .await
        .map_err(|e| OrigaError::RepositoryError {
            reason: format!("Failed to check cache: {:?}", e),
        })?;
    tracing::debug!(
        "Cache match checked ({:.2}s)",
        (now_ms() - match_start) / 1000.0
    );

    if response_option.is_undefined() || response_option.is_null() {
        return Ok(None);
    }

    let response: web_sys::Response = response_option
        .dyn_into()
        .map_err(|e| OrigaError::RepositoryError {
            reason: format!("Failed to cast Response: {:?}", e),
        })?;

    if !response.ok() {
        return Ok(None);
    }

    let _read_start = now_ms();
    let array_buffer_promise = response
        .array_buffer()
        .map_err(|e| OrigaError::RepositoryError {
            reason: format!("Failed to get array buffer: {:?}", e),
        })?;

    let array_buffer = JsFuture::from(array_buffer_promise)
        .await
        .map_err(|e| OrigaError::RepositoryError {
            reason: format!("Failed to read array buffer: {:?}", e),
        })?;

    let uint8_array = js_sys::Uint8Array::new(&array_buffer);
    let bytes = uint8_array.to_vec();
    tracing::info!(
        "Cache loaded: {} ({} bytes, {:.2}s)",
        url,
        bytes.len(),
        (now_ms() - start) / 1000.0
    );
    Ok(Some(bytes))
}

/// Save dictionary as rkyv bytes to cache
pub async fn save_dictionary_to_cache_rkyv(data: &DictionaryData) -> Result<(), OrigaError> {
    let start = now_ms();

    let serialize_start = now_ms();
    let bytes = origa::domain::serialize_dictionary_to_rkyv(data)
        .map_err(|e| OrigaError::RepositoryError {
            reason: format!("Failed to serialize dictionary: {:?}", e),
        })?;
    tracing::info!(
        "Dictionary serialized rkyv ({} bytes, {:.2}s)",
        bytes.len(),
        (now_ms() - serialize_start) / 1000.0
    );

    let cache = open_cache().await?;
    tracing::debug!("Cache opened for save ({:.2}s)", (now_ms() - start) / 1000.0);

    let array_buffer = js_sys::ArrayBuffer::new(bytes.len() as u32);
    let view = js_sys::Uint8Array::new(&array_buffer);
    view.copy_from(&bytes);

    let response_init = web_sys::ResponseInit::new();
    response_init.set_status(200);
    response_init.set_status_text("OK");

    let blob_property_bag = web_sys::BlobPropertyBag::new();
    blob_property_bag.set_type("application/octet-stream");

    let blob_parts = js_sys::Array::new();
    blob_parts.push(&array_buffer);

    let blob = web_sys::Blob::new_with_buffer_source_sequence_and_options(&blob_parts, &blob_property_bag)
        .map_err(|e| OrigaError::RepositoryError {
            reason: format!("Failed to create blob: {:?}", e),
        })?;

    let response = web_sys::Response::new_with_opt_blob_and_init(Some(&blob), &response_init)
        .map_err(|e| OrigaError::RepositoryError {
            reason: format!("Failed to create response: {:?}", e),
        })?;

    let url = urls().dictionary;
    let put_start = now_ms();
    let put_promise = cache.put_with_str(url, &response);
    JsFuture::from(put_promise)
        .await
        .map_err(|e| OrigaError::RepositoryError {
            reason: format!("Failed to save to cache: {:?}", e),
        })?;
    tracing::info!(
        "Cache saved: {} (put {:.2}s, total {:.2}s)",
        url,
        (now_ms() - put_start) / 1000.0,
        (now_ms() - start) / 1000.0
    );

    Ok(())
}
