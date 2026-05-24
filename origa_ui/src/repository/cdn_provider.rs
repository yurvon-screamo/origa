use std::collections::HashMap;
use std::future::Future;
use std::sync::Mutex;
use std::sync::OnceLock;

use leptos::task::spawn_local;
use leptos::wasm_bindgen::JsCast;
use origa::domain::OrigaError;
use origa::traits::CdnProvider;
use wasm_bindgen_futures::JsFuture;

use crate::core::config::cdn_url;

const CACHE_NAME: &str = "origa-cdn-v1";

pub struct CacheFirstCdnProvider;

static CDN_PROVIDER: OnceLock<CacheFirstCdnProvider> = OnceLock::new();

pub fn cdn() -> &'static CacheFirstCdnProvider {
    CDN_PROVIDER.get_or_init(|| CacheFirstCdnProvider)
}

static BLOB_URL_CACHE: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();

const MAX_BLOB_URL_CACHE_SIZE: usize = 500;

fn blob_url_cache() -> &'static Mutex<HashMap<String, String>> {
    BLOB_URL_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

async fn open_cache() -> Result<web_sys::Cache, OrigaError> {
    let window = web_sys::window().ok_or_else(|| OrigaError::RepositoryError {
        reason: "No window found".to_string(),
    })?;

    let caches = window.caches().map_err(|e| OrigaError::RepositoryError {
        reason: format!("Cache API not available: {:?}", e),
    })?;

    let cache =
        JsFuture::from(caches.open(CACHE_NAME))
            .await
            .map_err(|e| OrigaError::RepositoryError {
                reason: format!("Failed to open cache: {:?}", e),
            })?;

    cache.dyn_into().map_err(|e| OrigaError::RepositoryError {
        reason: format!("Failed to cast Cache: {:?}", e),
    })
}

fn ensure_leading_slash(path: &str) -> String {
    if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{}", path)
    }
}

async fn get_text_from_cache(cache: &web_sys::Cache, path: &str) -> Option<String> {
    let result = JsFuture::from(cache.match_with_str(path)).await.ok()?;
    if result.is_null() || result.is_undefined() {
        return None;
    }
    let response: web_sys::Response = result.dyn_into().ok()?;
    if !response.ok() {
        return None;
    }
    let text = JsFuture::from(response.text().ok()?).await.ok()?;
    text.as_string()
}

async fn get_bytes_from_cache(cache: &web_sys::Cache, path: &str) -> Option<Vec<u8>> {
    let result = JsFuture::from(cache.match_with_str(path)).await.ok()?;
    if result.is_null() || result.is_undefined() {
        return None;
    }
    let response: web_sys::Response = result.dyn_into().ok()?;
    if !response.ok() {
        return None;
    }
    let ab = JsFuture::from(response.array_buffer().ok()?).await.ok()?;
    let arr = js_sys::Uint8Array::new(&ab);
    Some(arr.to_vec())
}

async fn save_response_to_cache(
    cache: &web_sys::Cache,
    path: &str,
    response: &web_sys::Response,
) -> Result<(), OrigaError> {
    let request =
        web_sys::Request::new_with_str(path).map_err(|e| OrigaError::RepositoryError {
            reason: format!("Failed to create cache request: {:?}", e),
        })?;

    JsFuture::from(cache.put_with_request(&request, response))
        .await
        .map_err(|e| OrigaError::RepositoryError {
            reason: format!("Failed to save to cache: {:?}", e),
        })?;

    Ok(())
}

async fn fetch_text_from_cdn(path: &str) -> Result<(web_sys::Response, String), OrigaError> {
    let url = cdn_url(&ensure_leading_slash(path));

    let window = web_sys::window().ok_or_else(|| OrigaError::NetworkError {
        url: url.clone(),
        reason: "No window found".to_string(),
    })?;

    let resp_value = JsFuture::from(window.fetch_with_str(&url))
        .await
        .map_err(|e| OrigaError::NetworkError {
            url: url.clone(),
            reason: format!("Failed to fetch: {:?}", e),
        })?;

    let response: web_sys::Response =
        resp_value
            .dyn_into()
            .map_err(|e| OrigaError::NetworkError {
                url: url.clone(),
                reason: format!("Failed to cast response: {:?}", e),
            })?;

    if !response.ok() {
        return Err(OrigaError::NetworkError {
            url: url.clone(),
            reason: format!("HTTP {}", response.status()),
        });
    }

    let cloned = response.clone().map_err(|e| OrigaError::NetworkError {
        url: url.clone(),
        reason: format!("Failed to clone response: {:?}", e),
    })?;

    let text = JsFuture::from(cloned.text().map_err(|e| OrigaError::NetworkError {
        url: url.clone(),
        reason: format!("Failed to get text promise: {:?}", e),
    })?)
    .await
    .map_err(|e| OrigaError::NetworkError {
        url: url.clone(),
        reason: format!("Failed to read text: {:?}", e),
    })?;

    let text_str = text.as_string().ok_or_else(|| OrigaError::NetworkError {
        url: url.clone(),
        reason: "Response is not a string".to_string(),
    })?;

    Ok((response, text_str))
}

async fn fetch_bytes_from_cdn(path: &str) -> Result<(web_sys::Response, Vec<u8>), OrigaError> {
    let url = cdn_url(&ensure_leading_slash(path));

    let window = web_sys::window().ok_or_else(|| OrigaError::NetworkError {
        url: url.clone(),
        reason: "No window found".to_string(),
    })?;

    let resp_value = JsFuture::from(window.fetch_with_str(&url))
        .await
        .map_err(|e| OrigaError::NetworkError {
            url: url.clone(),
            reason: format!("Failed to fetch: {:?}", e),
        })?;

    let response: web_sys::Response =
        resp_value
            .dyn_into()
            .map_err(|e| OrigaError::NetworkError {
                url: url.clone(),
                reason: format!("Failed to cast response: {:?}", e),
            })?;

    if !response.ok() {
        return Err(OrigaError::NetworkError {
            url: url.clone(),
            reason: format!("HTTP {}", response.status()),
        });
    }

    let cloned = response.clone().map_err(|e| OrigaError::NetworkError {
        url: url.clone(),
        reason: format!("Failed to clone response: {:?}", e),
    })?;

    let ab = JsFuture::from(
        cloned
            .array_buffer()
            .map_err(|e| OrigaError::NetworkError {
                url: url.clone(),
                reason: format!("Failed to get array_buffer promise: {:?}", e),
            })?,
    )
    .await
    .map_err(|e| OrigaError::NetworkError {
        url: url.clone(),
        reason: format!("Failed to read array_buffer: {:?}", e),
    })?;

    let arr = js_sys::Uint8Array::new(&ab);
    Ok((response, arr.to_vec()))
}

impl CdnProvider for CacheFirstCdnProvider {
    fn fetch_text(&self, path: &str) -> impl Future<Output = Result<String, OrigaError>> {
        let path = path.to_string();
        async move {
            let cache = open_cache().await?;

            if let Some(text) = get_text_from_cache(&cache, &path).await {
                tracing::debug!(path = %path, "Cache hit (text)");
                return Ok(text);
            }

            tracing::debug!(path = %path, "Cache miss, fetching from CDN");
            let (response, text) = fetch_text_from_cdn(&path).await?;

            if let Err(e) = save_response_to_cache(&cache, &path, &response).await {
                tracing::warn!(path = %path, error = ?e, "Failed to cache text response");
            }

            Ok(text)
        }
    }

    fn fetch_bytes(&self, path: &str) -> impl Future<Output = Result<Vec<u8>, OrigaError>> {
        let path = path.to_string();
        async move {
            let cache = open_cache().await?;

            if let Some(bytes) = get_bytes_from_cache(&cache, &path).await {
                tracing::debug!(path = %path, "Cache hit (bytes)");
                return Ok(bytes);
            }

            tracing::debug!(path = %path, "Cache miss, fetching from CDN");
            let (response, bytes) = fetch_bytes_from_cdn(&path).await?;

            if let Err(e) = save_response_to_cache(&cache, &path, &response).await {
                tracing::warn!(path = %path, error = ?e, "Failed to cache bytes response");
            }

            Ok(bytes)
        }
    }
}

pub fn get_cached_blob_url(path: &str) -> Option<String> {
    let key = ensure_leading_slash(path);
    let cache = blob_url_cache().lock().ok()?;
    cache.get(&key).cloned()
}

pub async fn prefetch_blob_url(path: &str) -> Result<String, OrigaError> {
    let key = ensure_leading_slash(path);

    if let Some(blob_url) = get_cached_blob_url(&key) {
        return Ok(blob_url);
    }

    let bytes = cdn().fetch_bytes(&key).await?;

    let blob_options = web_sys::BlobPropertyBag::new();
    blob_options.set_type("audio/opus");

    let uint8_array = js_sys::Uint8Array::new_with_length(bytes.len() as u32);
    uint8_array.copy_from(&bytes);
    let parts = js_sys::Array::of1(&uint8_array);
    let blob = web_sys::Blob::new_with_u8_array_sequence_and_options(&parts, &blob_options)
        .map_err(|e| OrigaError::RepositoryError {
            reason: format!("Failed to create audio Blob: {:?}", e),
        })?;

    let blob_url = web_sys::Url::create_object_url_with_blob(&blob).map_err(|e| {
        OrigaError::RepositoryError {
            reason: format!("Failed to create blob URL: {:?}", e),
        }
    })?;

    {
        let mut cache = blob_url_cache().lock().map_err(|e| OrigaError::RepositoryError {
            reason: format!("Failed to lock blob URL cache: {:?}", e),
        })?;
        if cache.len() < MAX_BLOB_URL_CACHE_SIZE {
            cache.insert(key, blob_url.clone());
        }
    }

    Ok(blob_url)
}

pub fn resolve_audio_url(path: &str) -> String {
    let key = ensure_leading_slash(path);

    if let Some(blob_url) = get_cached_blob_url(&key) {
        return blob_url;
    }

    let path_owned = key.clone();
    spawn_local(async move {
        if let Err(e) = prefetch_blob_url(&path_owned).await {
            tracing::warn!(path = %path_owned, error = ?e, "Failed to prefetch blob URL");
        }
    });

    cdn_url(&key)
}
