use std::future::Future;
use std::sync::OnceLock;

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
