use serde::de::DeserializeOwned;
use serde::Serialize;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;

use origa::domain::{
    DictionaryData, GrammarData, KanjiData, OrigaError, RadicalData, VocabularyChunkData,
};

const CACHE_NAME: &str = "origa-dictionary-v1";

const DICTIONARY_URL: &str = "/public/dictionaries/unidic/cache/dictionary-data";
const VOCABULARY_URL: &str = "/public/domain/dictionary/vocabulary/cache/vocabulary-data";
const KANJI_URL: &str = "/public/domain/dictionary/kanji/cache/kanji-data";
const RADICAL_URL: &str = "/public/domain/dictionary/radical/cache/radical-data";
const GRAMMAR_URL: &str = "/public/domain/grammar/cache/grammar-data";

async fn open_cache() -> Result<web_sys::Cache, OrigaError> {
    let window = web_sys::window().ok_or_else(|| OrigaError::RepositoryError {
        reason: "No window found".to_string(),
    })?;

    let caches = js_sys::Reflect::get(&window, &wasm_bindgen::JsValue::from_str("caches"))
        .map_err(|e| OrigaError::RepositoryError {
            reason: format!("Cache API not available: {:?}", e),
        })?;

    let caches: web_sys::CacheStorage = caches.dyn_into().map_err(|e| OrigaError::RepositoryError {
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
    let response_option = JsFuture::from(match_promise)
        .await
        .map_err(|e| OrigaError::RepositoryError {
            reason: format!("Failed to check cache: {:?}", e),
        })?;

    if response_option.is_undefined() || response_option.is_null() {
        return Ok(None);
    }

    let response: web_sys::Response = response_option.dyn_into().map_err(|e| OrigaError::RepositoryError {
        reason: format!("Failed to cast Response: {:?}", e),
    })?;

    if !response.ok() {
        return Ok(None);
    }

    let array_buffer_promise = response.array_buffer().map_err(|e| OrigaError::RepositoryError {
        reason: format!("Failed to get array buffer: {:?}", e),
    })?;

    let array_buffer = JsFuture::from(array_buffer_promise)
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
    get_cached_data(DICTIONARY_URL).await
}

pub async fn save_dictionary_to_cache(data: &DictionaryData) -> Result<(), OrigaError> {
    save_data_to_cache(DICTIONARY_URL, data).await
}

pub async fn get_cached_vocabulary() -> Result<Option<VocabularyChunkData>, OrigaError> {
    get_cached_data(VOCABULARY_URL).await
}

pub async fn save_vocabulary_to_cache(data: &VocabularyChunkData) -> Result<(), OrigaError> {
    save_data_to_cache(VOCABULARY_URL, data).await
}

pub async fn get_cached_kanji() -> Result<Option<KanjiData>, OrigaError> {
    get_cached_data(KANJI_URL).await
}

pub async fn save_kanji_to_cache(data: &KanjiData) -> Result<(), OrigaError> {
    save_data_to_cache(KANJI_URL, data).await
}

pub async fn get_cached_radical() -> Result<Option<RadicalData>, OrigaError> {
    get_cached_data(RADICAL_URL).await
}

pub async fn save_radical_to_cache(data: &RadicalData) -> Result<(), OrigaError> {
    save_data_to_cache(RADICAL_URL, data).await
}

pub async fn get_cached_grammar() -> Result<Option<GrammarData>, OrigaError> {
    get_cached_data(GRAMMAR_URL).await
}

pub async fn save_grammar_to_cache(data: &GrammarData) -> Result<(), OrigaError> {
    save_data_to_cache(GRAMMAR_URL, data).await
}
