use origa::domain::{DictionaryData, OrigaError};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;

const CACHE_NAME: &str = "origa-dictionary-v1";
const DATA_URL: &str = "/public/dictionaries/unidic/cache/dictionary-data";

pub async fn get_cached_dictionary() -> Result<Option<DictionaryData>, OrigaError> {
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

    let match_promise = cache.match_with_str(DATA_URL);
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

    let data: DictionaryData =
        bincode::deserialize(&bytes).map_err(|e| OrigaError::RepositoryError {
            reason: format!("Failed to deserialize dictionary: {:?}", e),
        })?;

    web_sys::console::info_1(&"Dictionary loaded from Cache API".into());
    Ok(Some(data))
}

pub async fn save_dictionary_to_cache(data: &DictionaryData) -> Result<(), OrigaError> {
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

    let bytes = bincode::serialize(data).map_err(|e| OrigaError::RepositoryError {
        reason: format!("Failed to serialize dictionary: {:?}", e),
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

    let put_promise = cache.put_with_str(DATA_URL, &response);
    JsFuture::from(put_promise)
        .await
        .map_err(|e| OrigaError::RepositoryError {
            reason: format!("Failed to save to cache: {:?}", e),
        })?;

    web_sys::console::info_1(&"Dictionary saved to Cache API".into());
    Ok(())
}
