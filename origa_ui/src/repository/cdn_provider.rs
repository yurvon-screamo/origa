use std::collections::HashMap;
use std::collections::VecDeque;
use std::future::Future;
use std::sync::Mutex;
use std::sync::OnceLock;

use leptos::wasm_bindgen::JsCast;
use origa::domain::OrigaError;
use origa::traits::CdnProvider;
use wasm_bindgen_futures::JsFuture;

use crate::core::config::cdn_url;

pub const CDN_CACHE_NAME: &str = "origa-cdn-v1";

/// Outcome of the cache-first policy for one resource (ADR-053).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FetchDecision {
    ServeFromCache,
    FetchFromNetwork,
    FailOffline,
}

/// Pure form of the provider policy: a cache hit always wins; a miss
/// goes to the network unless the CDN was already proven unreachable
/// this run (instant refusal instead of a doomed request).
pub fn fetch_decision(cache_hit: bool, cdn_unreachable: bool) -> FetchDecision {
    if cache_hit {
        FetchDecision::ServeFromCache
    } else if cdn_unreachable {
        FetchDecision::FailOffline
    } else {
        FetchDecision::FetchFromNetwork
    }
}

/// Process-global "the CDN is not reachable" verdict. Set by the
/// startup manifest probe / a stalled manifest fetch; cleared by Retry
/// and by the browser `online` event. A false positive only costs one
/// extra attempt after the clear; a false negative hangs the app —
/// which is why it is never set from `navigator.onLine` alone.
static CDN_UNREACHABLE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

#[cfg(any(target_arch = "wasm32", test))]
pub fn mark_cdn_unreachable() {
    CDN_UNREACHABLE.store(true, std::sync::atomic::Ordering::Release);
}

pub fn is_cdn_unreachable() -> bool {
    CDN_UNREACHABLE.load(std::sync::atomic::Ordering::Acquire)
}

pub fn clear_cdn_unreachable() {
    CDN_UNREACHABLE.store(false, std::sync::atomic::Ordering::Release);
}

/// Marker for the instant refusal errors produced under the
/// unreachable flag (distinct from idle timeouts and HTTP failures).
/// Consumed by the retry gate (`should_retry_after_error`).
pub struct CdnUnreachableError;

impl CdnUnreachableError {
    pub fn is_match(error: &OrigaError) -> bool {
        matches!(
            error,
            OrigaError::NetworkError { reason, .. } if reason.starts_with(UNREACHABLE_MARKER)
        )
    }
}

const UNREACHABLE_MARKER: &str = "cdn unreachable";

fn cdn_unreachable_error(url: &str) -> OrigaError {
    OrigaError::NetworkError {
        url: url.to_string(),
        reason: format!("{UNREACHABLE_MARKER}: skipping request after probe verdict"),
    }
}

pub struct CacheFirstCdnProvider;

static CDN_PROVIDER: OnceLock<CacheFirstCdnProvider> = OnceLock::new();

pub fn cdn() -> &'static CacheFirstCdnProvider {
    CDN_PROVIDER.get_or_init(|| CacheFirstCdnProvider)
}

static BLOB_URL_CACHE: OnceLock<Mutex<BlobUrlCache>> = OnceLock::new();

/// Ceiling on the number of retained `blob:` URLs. Each entry holds an
/// underlying `Blob` + `ArrayBuffer` alive in JS heap until the URL is
/// `revoke_object_url`'d. Bounded to keep memory predictable on low-end
/// Android WebViews: ~500 phrases × ~3-50 KB/opus ≈ 1.5-25 MB ceiling. When
/// the limit is reached the oldest entry is evicted and its blob URL revoked.
const MAX_BLOB_URL_CACHE_SIZE: usize = 500;

/// Insertion-ordered blob URL cache with bounded LRU-ish (FIFO) eviction.
/// Tracks keys in `order` so the oldest materialised URL can be revoked when
/// the cap is reached, preventing unbounded `Blob`/`ArrayBuffer` leaks.
struct BlobUrlCache {
    urls: HashMap<String, String>,
    order: VecDeque<String>,
}

impl BlobUrlCache {
    fn get(&self, key: &str) -> Option<&String> {
        self.urls.get(key)
    }

    fn insert(&mut self, key: String, url: String) {
        if self.urls.insert(key.clone(), url).is_some() {
            return;
        }
        while self.order.len() >= MAX_BLOB_URL_CACHE_SIZE {
            let Some(evicted_key) = self.order.pop_front() else {
                break;
            };
            if let Some(evicted_url) = self.urls.remove(&evicted_key) {
                let _ = web_sys::Url::revoke_object_url(&evicted_url);
            }
        }
        self.order.push_back(key);
    }

    #[cfg(test)]
    fn remove(&mut self, key: &str) {
        if self.urls.remove(key).is_some() {
            self.order.retain(|k| k != key);
        }
    }
}

fn blob_url_cache() -> &'static Mutex<BlobUrlCache> {
    BLOB_URL_CACHE.get_or_init(|| {
        Mutex::new(BlobUrlCache {
            urls: HashMap::new(),
            order: VecDeque::new(),
        })
    })
}

async fn open_cache() -> Result<web_sys::Cache, OrigaError> {
    let window = web_sys::window().ok_or_else(|| OrigaError::RepositoryError {
        reason: "No window found".to_string(),
    })?;

    let caches = window.caches().map_err(|e| OrigaError::RepositoryError {
        reason: format!("Cache API not available: {:?}", e),
    })?;

    let cache = JsFuture::from(caches.open(CDN_CACHE_NAME))
        .await
        .map_err(|e| OrigaError::RepositoryError {
            reason: format!("Failed to open cache: {:?}", e),
        })?;

    cache.dyn_into().map_err(|e| OrigaError::RepositoryError {
        reason: format!("Failed to cast Cache: {:?}", e),
    })
}

/// Store a simple marker value in the CDN cache.
pub async fn store_cache_marker(key: &str, value: &str) -> Result<(), OrigaError> {
    let cache = open_cache().await?;
    let response = web_sys::Response::new_with_opt_str(Some(value)).map_err(|e| {
        OrigaError::RepositoryError {
            reason: format!("Failed to create marker response: {:?}", e),
        }
    })?;
    let url = cdn_cache_url(key);
    let request =
        web_sys::Request::new_with_str(&url).map_err(|e| OrigaError::RepositoryError {
            reason: format!("Failed to create marker request: {:?}", e),
        })?;
    JsFuture::from(cache.put_with_request(&request, &response))
        .await
        .map_err(|e| OrigaError::RepositoryError {
            reason: format!("Failed to store marker: {:?}", e),
        })?;
    Ok(())
}

fn ensure_leading_slash(path: &str) -> String {
    if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{}", path)
    }
}

/// Absolute URL used as the Cache API key for `path`.
///
/// The Cache API rejects request URLs that are not HTTP(S): a relative path
/// resolves against the WebView origin (`tauri://localhost` on iOS and desktop
/// production builds) and WebKit throws `Request url is not HTTP/HTTPS`. Keys
/// are therefore always built on the full CDN base.
///
/// Unlike [`crate::core::config::cdn_url`], preserves any `?v=` query string —
/// phrase chunks and versioned content use it as a content hash, so it must be
/// part of the cache key or a content change would keep serving the stale entry.
pub(crate) fn cdn_cache_url(path: &str) -> String {
    let base = env!("ORIGA_CDN_BASE_URL").trim_end_matches('/');
    format!("{}{}", base, ensure_leading_slash(path))
}

async fn get_text_from_cache(cache: &web_sys::Cache, path: &str) -> Option<String> {
    let url = cdn_cache_url(path);
    let result = JsFuture::from(cache.match_with_str(&url)).await.ok()?;
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
    let url = cdn_cache_url(path);
    let result = JsFuture::from(cache.match_with_str(&url)).await.ok()?;
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
    let url = cdn_cache_url(path);
    let request =
        web_sys::Request::new_with_str(&url).map_err(|e| OrigaError::RepositoryError {
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
    crate::utils::net_timeout::fetch_text_idle(&url).await
}

async fn fetch_bytes_from_cdn(path: &str) -> Result<(web_sys::Response, Vec<u8>), OrigaError> {
    let url = cdn_url(&ensure_leading_slash(path));
    crate::utils::net_timeout::fetch_bytes_idle(&url).await
}

impl CdnProvider for CacheFirstCdnProvider {
    fn fetch_text(&self, path: &str) -> impl Future<Output = Result<String, OrigaError>> {
        let path = path.to_string();
        async move {
            let cache = open_cache().await?;

            let cache_hit = get_text_from_cache(&cache, &path).await;
            if fetch_decision(cache_hit.is_some(), is_cdn_unreachable())
                == FetchDecision::FailOffline
            {
                tracing::debug!(path = %path, "Cache miss, CDN unreachable — refusing");
                return Err(cdn_unreachable_error(&path));
            }
            if let Some(text) = cache_hit {
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

            let cache_hit = get_bytes_from_cache(&cache, &path).await;
            if fetch_decision(cache_hit.is_some(), is_cdn_unreachable())
                == FetchDecision::FailOffline
            {
                tracing::debug!(path = %path, "Cache miss, CDN unreachable — refusing");
                return Err(cdn_unreachable_error(&path));
            }
            if let Some(bytes) = cache_hit {
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

/// Removes one entry from the CDN cache. Used by content-specific
/// self-heal paths (kanji art / JLPT bundles): a poisoned cache body
/// (HTTP 200 the origin never produced) never revalidates under the
/// cache-first policy, so the healer purges the entry explicitly. The
/// purge runs BEFORE the network refetch so the poison is gone from the
/// cache even when the refetch fails offline — the next read is then an
/// honest cache miss instead of the same garbage.
pub async fn purge_entry(path: &str) -> Result<(), OrigaError> {
    let cache = open_cache().await?;
    JsFuture::from(cache.delete_with_str(&cdn_cache_url(path)))
        .await
        .map_err(|e| OrigaError::RepositoryError {
            reason: format!("Failed to delete cache entry: {e:?}"),
        })?;
    Ok(())
}

/// Network-only text fetch that overwrites the cache entry with the
/// fresh body — the refetch half of the self-heal pair above. The
/// unreachable guard mirrors `FetchDecision::FailOffline`: a CDN proven
/// unreachable must refuse instantly instead of hanging every healed
/// kanji on the full idle deadline.
pub async fn fetch_text_bypassing_cache(path: &str) -> Result<String, OrigaError> {
    if is_cdn_unreachable() {
        tracing::debug!(path = %path, "Bypass fetch refused — CDN unreachable");
        return Err(cdn_unreachable_error(path));
    }

    let (response, text) = fetch_text_from_cdn(path).await?;

    let cache = open_cache().await?;
    if let Err(e) = save_response_to_cache(&cache, path, &response).await {
        tracing::warn!(path = %path, error = ?e, "Failed to cache refreshed text response");
    }

    Ok(text)
}

/// Verdict of comparing a cache-served text body against the manifest
/// hash of its path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextFreshness {
    /// Body hash equals the manifest hash — the cache is current.
    Verified,
    /// The manifest tracks the file and the hash differs — stale body.
    Stale,
    /// No manifest hash available (native build, offline start, untracked
    /// path) — unverifiable, the cache is trusted at the usual level.
    Unverified,
}

/// Pure decision core of [`fetch_text_verified`]: sha256-hex of the body
/// vs the manifest hash (lowercase hex, as the deploy manifest stores it).
/// The comparison is ASCII-case-insensitive so a future producer emitting
/// uppercase hex cannot turn every fetch into a perpetual divergence.
pub fn text_freshness(text: &str, manifest_hash: Option<&str>) -> TextFreshness {
    let Some(expected) = manifest_hash else {
        return TextFreshness::Unverified;
    };
    let actual: String = origa::dictionary::cdn_blob::sha256_bytes(text.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    if actual.eq_ignore_ascii_case(expected) {
        TextFreshness::Verified
    } else {
        TextFreshness::Stale
    }
}

/// Injectable orchestration of [`fetch_text_verified`]: cache-first text
/// from any [`CdnProvider`], freshness verified against `manifest_hash`,
/// fresh refetch delegated to the `refetch` closure (the real one
/// overwrites the cache entry; tests inject a stub — the production bypass
/// needs the browser Cache API and cannot run in native tests).
///
/// Fail-open contract: a refetch failure keeps the already-fetched (stale)
/// body. That is exactly what callers got before this heal existed — an
/// unreachable network must degrade to a working stale resource, not
/// convert it into a loader error. The divergence persists and the next
/// start re-attempts the heal. A refetched body that STILL diverges is
/// served too (best available) and logged — an edge/origin persistently
/// serving divergent content stays visible in the logs instead of
/// silently looping detect→refetch on every start.
///
/// `Unverified` (no manifest hash — native build, offline start, untracked
/// path) intentionally trusts the cache. This is the opposite verdict of
/// the blob-guard family (`GuardExpectation::Mismatch` for a missing
/// path) and cannot be unified: an rkyv blob carries its guard in the
/// header, so an untracked path IS detectably anomalous; a plain text body
/// has nothing to compare against a manifest entry that does not exist —
/// trusting the cache is the only coherent option.
///
/// The hash cost is paid per call — keep this for kilobyte-scale content
/// files, not multi-megabyte blobs.
async fn resolve_verified_text<P, F, Fut>(
    cdn: &P,
    path: &str,
    manifest_hash: Option<&str>,
    refetch: F,
) -> Result<String, OrigaError>
where
    P: CdnProvider,
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<String, OrigaError>>,
{
    let text = cdn.fetch_text(path).await?;
    match text_freshness(&text, manifest_hash) {
        TextFreshness::Verified | TextFreshness::Unverified => Ok(text),
        TextFreshness::Stale => match refetch().await {
            Ok(fresh) => {
                if text_freshness(&fresh, manifest_hash) == TextFreshness::Stale {
                    tracing::warn!(
                        path = %path,
                        "Refetched body still diverges from manifest — serving it anyway"
                    );
                }
                Ok(fresh)
            },
            Err(e) => {
                tracing::warn!(
                    path = %path,
                    error = ?e,
                    "Cached body diverges from manifest and refetch failed — keeping the stale body"
                );
                Ok(text)
            },
        },
    }
}

/// Cache-first text fetch with manifest verification. A cache hit whose
/// body hash diverges from the remote manifest is refetched over the
/// network, overwriting the cache entry (via [`fetch_text_bypassing_cache`]).
///
/// The divergence is reachable for installs whose cache predates the
/// file's manifest tracking: the manifest diff cannot invalidate those
/// (cache_manager::find_stale_entries walks local entries only, and the
/// first post-deploy run already synced the local manifest). Without the
/// check a body edited on the CDN would serve stale forever — the stale
/// counter readings bug (uwuwu origa mJvjEgSqp0).
pub async fn fetch_text_verified(path: &str) -> Result<String, OrigaError> {
    let manifest_hash = super::cache_manager::remote_hash_for(path);
    resolve_verified_text(cdn(), path, manifest_hash, || {
        fetch_text_bypassing_cache(path)
    })
    .await
}

pub fn get_cached_blob_url(path: &str) -> Option<String> {
    let key = ensure_leading_slash(path);
    let cache = blob_url_cache().lock().ok()?;
    cache.get(&key).cloned()
}

/// Derive the Blob MIME type from the CDN path extension. Hardcoding
/// `audio/opus` couples decoding to a CDN filename convention; if a non-opus
/// record ever lands on the CDN the decoder would break silently. Falling back
/// to `audio/opus` matches the current CDN content while keeping other formats
/// decodeable if they appear.
fn mime_type_for_path(path: &str) -> &'static str {
    let ext = path.rsplit('.').next().map(str::to_ascii_lowercase);
    match ext.as_deref() {
        Some("opus") => "audio/opus",
        Some("mp3") => "audio/mpeg",
        Some("wav") => "audio/wav",
        Some("ogg") | Some("oga") => "audio/ogg",
        Some("m4a") | Some("mp4") | Some("aac") => "audio/mp4",
        Some("webm") => "audio/webm",
        _ => "audio/opus",
    }
}

pub async fn prefetch_blob_url(path: &str) -> Result<String, OrigaError> {
    let key = ensure_leading_slash(path);

    if let Some(blob_url) = get_cached_blob_url(&key) {
        return Ok(blob_url);
    }

    let bytes = cdn().fetch_bytes(&key).await?;

    let blob_options = web_sys::BlobPropertyBag::new();
    blob_options.set_type(mime_type_for_path(&key));

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
        let mut cache = blob_url_cache()
            .lock()
            .map_err(|e| OrigaError::RepositoryError {
                reason: format!("Failed to lock blob URL cache: {:?}", e),
            })?;
        cache.insert(key, blob_url.clone());
    }

    Ok(blob_url)
}

/// Resolve a playable audio URL for the given CDN path.
///
/// # Bug A root cause (see module-level `tests` doc)
///
/// The Railway Hikari reverse proxy gzip-encodes `.opus` responses on-the-fly,
/// and Chromium's `<audio>` element does not apply transport decompression —
/// feeding the raw gzip bytes to the Opus decoder yields
/// `ERR_CONTENT_DECODING_FAILED`. The only correct way to feed `<audio>` is
/// therefore a `blob:` URL produced from `fetch()`-decompressed bytes.
///
/// # Contract
///
/// Returns `Some(blob_url)` only when the blob URL has already been materialised
/// in the in-memory cache; returns `None` otherwise. Callers MUST NOT
/// substitute the CDN URL when this returns `None`: they must either await
/// `prefetch_blob_url` directly (preferred for playback paths) or fall back to
/// TTS. This function is a **pure synchronous lookup** with no side effects —
/// callers own the async prefetch, which avoids a double-fetch race when a
/// consumer needs the blob URL reactively.
pub fn resolve_audio_url(path: &str) -> Option<String> {
    let key = ensure_leading_slash(path);
    get_cached_blob_url(&key)
}

pub async fn prefetch_to_cache(path: &str) -> Result<(), OrigaError> {
    let cache = open_cache().await?;
    let key = ensure_leading_slash(path);

    let existing = JsFuture::from(cache.match_with_str(&cdn_cache_url(&key)))
        .await
        .map_err(|e| OrigaError::RepositoryError {
            reason: format!("Cache match failed: {:?}", e),
        })?;
    if !existing.is_null() && !existing.is_undefined() {
        return Ok(());
    }

    let url = cdn_url(&key);
    // Idle-deadline fetch (ADR-053): a dead network must not hang a
    // prefetch. The rebuilt response keeps the content type, so cached
    // audio still decodes via blob: URLs.
    let fetched = crate::utils::net_timeout::fetch_idle(
        &url,
        crate::utils::net_timeout::DEFAULT_IDLE_TIMEOUT_MS,
    )
    .await?;

    let request = web_sys::Request::new_with_str(&cdn_cache_url(&key)).map_err(|e| {
        OrigaError::RepositoryError {
            reason: format!("Failed to create request: {:?}", e),
        }
    })?;

    JsFuture::from(cache.put_with_request(&request, &fetched.response))
        .await
        .map_err(|e| OrigaError::RepositoryError {
            reason: format!("Failed to cache: {:?}", e),
        })?;

    Ok(())
}

pub async fn is_cached(path: &str) -> bool {
    let Ok(cache) = open_cache().await else {
        return false;
    };
    let key = ensure_leading_slash(path);
    let Ok(result) = JsFuture::from(cache.match_with_str(&cdn_cache_url(&key))).await else {
        return false;
    };
    !result.is_null() && !result.is_undefined()
}

/// Store arbitrary text in the CDN Cache API under the given path.
/// Used by bundle extraction: download a bundle, parse it, then store
/// individual chunks in cache so that lazy readers (phrase_data_loader)
/// get cache hits without individual CDN requests.
pub async fn store_text_in_cache(path: &str, text: &str) -> Result<(), OrigaError> {
    let cache = open_cache().await?;
    let key = ensure_leading_slash(path);
    let response = web_sys::Response::new_with_opt_str(Some(text)).map_err(|e| {
        OrigaError::RepositoryError {
            reason: format!("Failed to create response for cache store: {:?}", e),
        }
    })?;
    let request = web_sys::Request::new_with_str(&cdn_cache_url(&key)).map_err(|e| {
        OrigaError::RepositoryError {
            reason: format!("Failed to create request: {:?}", e),
        }
    })?;
    JsFuture::from(cache.put_with_request(&request, &response))
        .await
        .map_err(|e| OrigaError::RepositoryError {
            reason: format!("Failed to store in cache: {:?}", e),
        })?;
    Ok(())
}

#[cfg(all(target_arch = "wasm32", test))]
#[path = "cdn_offline_wasm_tests.rs"]
mod cdn_offline_wasm_tests;

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use std::cell::RefCell;

    /// Recording stub provider: serves a canned text and counts
    /// `fetch_text` calls (external CDN boundary — a mock is appropriate).
    struct StubCdn {
        body: String,
        calls: RefCell<Vec<String>>,
    }

    impl StubCdn {
        fn serving(body: &str) -> Self {
            Self {
                body: body.to_string(),
                calls: RefCell::new(Vec::new()),
            }
        }

        fn requested_paths(&self) -> Vec<String> {
            self.calls.borrow().clone()
        }
    }

    impl CdnProvider for StubCdn {
        fn fetch_text(&self, path: &str) -> impl Future<Output = Result<String, OrigaError>> {
            let path = path.to_string();
            async move {
                self.calls.borrow_mut().push(path);
                Ok(self.body.clone())
            }
        }

        async fn fetch_bytes(&self, _path: &str) -> Result<Vec<u8>, OrigaError> {
            Err(OrigaError::NetworkError {
                url: "stub".to_string(),
                reason: "not implemented".to_string(),
            })
        }
    }

    fn fresh_refetch(
        body: &'static str,
    ) -> impl FnOnce() -> futures::future::Ready<Result<String, OrigaError>> {
        move || futures::future::ready(Ok(body.to_string()))
    }

    fn failed_refetch() -> impl FnOnce() -> futures::future::Ready<Result<String, OrigaError>> {
        move || {
            futures::future::ready(Err(OrigaError::NetworkError {
                url: "refetch".to_string(),
                reason: "HTTP 503".to_string(),
            }))
        }
    }

    const STALE_BODY: &str = "stale registry";
    const STALE_HASH: &str = "1111111111111111111111111111111111111111111111111111111111111111";
    const FRESH_BODY: &str = "fresh registry";

    #[tokio::test]
    async fn diverged_cache_body_refetches_fresh_from_cdn() {
        // Arrange: a stale body in the cache, a manifest hash for the fixed
        // file, a working fresh refetch.
        let cdn = StubCdn::serving(STALE_BODY);

        // Act
        let resolved = resolve_verified_text(
            &cdn,
            "counters/counters.json",
            Some(STALE_HASH),
            fresh_refetch(FRESH_BODY),
        )
        .await
        .unwrap();

        // Assert
        assert_eq!(resolved, FRESH_BODY);
        assert_eq!(cdn.requested_paths(), vec!["counters/counters.json"]);
    }

    #[tokio::test]
    async fn failed_refetch_keeps_stale_body_without_error() {
        // Arrange: a diverged body and a failing network — the pre-heal
        // behavior for this user was "working stale data", and the heal
        // must not turn it into a loader error.
        let cdn = StubCdn::serving(STALE_BODY);

        // Act
        let resolved = resolve_verified_text(
            &cdn,
            "counters/counters.json",
            Some(STALE_HASH),
            failed_refetch(),
        )
        .await
        .unwrap();

        // Assert
        assert_eq!(resolved, STALE_BODY);
    }

    #[tokio::test]
    async fn refetched_body_that_still_diverges_is_served() {
        // Arrange: even the fresh network answer diverges (edge poisoning
        // class) — the best available body is still served, the divergence
        // is only logged.
        let cdn = StubCdn::serving(STALE_BODY);

        // Act
        let resolved = resolve_verified_text(
            &cdn,
            "counters/counters.json",
            Some(STALE_HASH),
            fresh_refetch(STALE_BODY),
        )
        .await
        .unwrap();

        // Assert
        assert_eq!(resolved, STALE_BODY);
    }

    #[tokio::test]
    async fn verified_cache_body_skips_the_refetch() {
        // Arrange: body hash matches the manifest — the refetch closure
        // panics if invoked (zero-call assertion via panic = the refetch
        // must not run on the happy path).
        let body = "current registry";
        let hash: String = origa::dictionary::cdn_blob::sha256_bytes(body.as_bytes())
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        let cdn = StubCdn::serving(body);

        // Act
        let resolved = resolve_verified_text(
            &cdn,
            "counters/counters.json",
            Some(&hash),
            || -> futures::future::Ready<Result<String, OrigaError>> {
                panic!("refetch must not run for a verified body")
            },
        )
        .await
        .unwrap();

        // Assert
        assert_eq!(resolved, body);
    }

    #[tokio::test]
    async fn unverified_body_without_manifest_hash_skips_the_refetch() {
        // Arrange: no manifest available (offline start / native build) —
        // the cache is trusted at the pre-heal level.
        let cdn = StubCdn::serving(STALE_BODY);

        // Act
        let resolved = resolve_verified_text(
            &cdn,
            "counters/counters.json",
            None,
            || -> futures::future::Ready<Result<String, OrigaError>> {
                panic!("refetch must not run without a manifest hash")
            },
        )
        .await
        .unwrap();

        // Assert
        assert_eq!(resolved, STALE_BODY);
    }

    // text_freshness: the pure decision core of the self-heal in
    // fetch_text_verified — a body matching the manifest hash is current,
    // a tracked file with a divergent hash is stale, and a missing hash
    // (offline start, untracked path) leaves the cache trusted.
    #[rstest]
    #[case::hash_matches_body_is_verified(
        "abc",
        Some("ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"),
        TextFreshness::Verified
    )]
    #[case::hash_divergence_is_stale("abc", Some("deadbeef"), TextFreshness::Stale)]
    #[case::missing_hash_is_unverified("abc", None, TextFreshness::Unverified)]
    fn text_freshness_classifies_body_against_manifest_hash(
        #[case] body: &str,
        #[case] manifest_hash: Option<&str>,
        #[case] expected: TextFreshness,
    ) {
        assert_eq!(text_freshness(body, manifest_hash), expected);
    }

    // Bug A — root cause (proven via curl by the architect):
    //
    // Railway's Hikari reverse proxy applies `Content-Encoding: gzip` to `.opus`
    // responses on-the-fly whenever the client advertises `Accept-Encoding` (every
    // browser / WebView does this automatically; it is a "forbidden header name"
    // and cannot be disabled from JS).
    //
    //   curl -H 'Accept-Encoding: identity'  → no Content-Encoding, 3139 bytes ✅
    //   curl -H 'Accept-Encoding: gzip'      → Content-Encoding: gzip, 3145 bytes ❌
    //   curl -H 'Accept-Encoding: br'        → Content-Encoding: gzip (!), 3145 bytes ❌
    //
    // Gzip makes the file LARGER (3145 > 3139) — negative compression for an
    // already-compressed Opus stream. Chromium's <audio> element historically
    // does not apply transport-layer decompression (crbug 1029876 / 596361 /
    // 1247420), so it hands the raw gzip bytes to the media decoder which then
    // fails with `ERR_CONTENT_DECODING_FAILED`.
    //
    // The fix is "always-blob-URL playback": the Fetch API transparently
    // decompresses `Content-Encoding`, after which the resulting bytes are
    // stored in a Blob. The `blob:` URL handed to <audio> therefore has no
    // transport encoding and decodes correctly.
    //
    // As a consequence, `resolve_audio_url` must NEVER return the raw CDN URL
    // as a synchronous fallback. The contract below is a compile-time
    // regression guard: callers must receive `Option<String>` and either await
    // the prefetched blob URL or fall back to TTS — never play the CDN URL.

    /// Compile-time contract: `resolve_audio_url` returns `Option<String>`.
    ///
    /// Before Slice-2 this function returned `String` (the CDN URL when the
    /// blob URL had not been prefetched yet). That is precisely the code path
    /// that triggered `ERR_CONTENT_DECODING_FAILED`. If this test fails to
    /// compile, the contract has regressed.
    #[test]
    fn resolve_audio_url_contract_returns_option() {
        let _type_witness: fn(&str) -> Option<String> = resolve_audio_url;
        // Touch the symbol so the assignment is not dead-code eliminated.
        let _ = _type_witness as fn(&str) -> Option<String>;
    }

    #[test]
    fn unreachable_flag_round_trips_between_mark_and_clear() {
        clear_cdn_unreachable();
        assert!(!is_cdn_unreachable());

        mark_cdn_unreachable();
        assert!(is_cdn_unreachable());

        clear_cdn_unreachable();
        assert!(!is_cdn_unreachable());
    }

    #[test]
    fn unreachable_refusal_marker_is_recognizable() {
        let refusal = OrigaError::NetworkError {
            url: "probe".to_string(),
            reason: "cdn unreachable: skipping request after probe verdict".to_string(),
        };
        assert!(CdnUnreachableError::is_match(&refusal));

        let stalled = OrigaError::NetworkError {
            url: "probe".to_string(),
            reason: "idle timeout after 10000 ms without data".to_string(),
        };
        assert!(!CdnUnreachableError::is_match(&stalled));

        let http = OrigaError::NetworkError {
            url: "probe".to_string(),
            reason: "HTTP 503".to_string(),
        };
        assert!(!CdnUnreachableError::is_match(&http));
    }

    #[tokio::test]
    async fn bypass_refetch_refuses_instantly_when_cdn_unreachable() {
        // The guard clears the global flag on ANY exit path, so a failing
        // assert cannot leak the verdict into parallel/later tests.
        struct UnreachableGuard;
        impl Drop for UnreachableGuard {
            fn drop(&mut self) {
                clear_cdn_unreachable();
            }
        }
        clear_cdn_unreachable();
        mark_cdn_unreachable();
        let _guard = UnreachableGuard;

        let started = std::time::Instant::now();
        let error = fetch_text_bypassing_cache("/heal-probe/never-cached.json")
            .await
            .expect_err("unreachable CDN must refuse the bypass fetch");

        assert!(CdnUnreachableError::is_match(&error), "got: {error:?}");
        assert!(
            started.elapsed().as_millis() < 900,
            "the refusal must be instant — no idle-deadline wait"
        );
    }

    #[test]
    fn fetch_decision_matrix_prefers_cache_and_refuses_dead_network() {
        use super::FetchDecision::*;

        assert_eq!(fetch_decision(true, false), ServeFromCache);
        assert_eq!(fetch_decision(true, true), ServeFromCache);
        assert_eq!(fetch_decision(false, false), FetchFromNetwork);
        assert_eq!(fetch_decision(false, true), FailOffline);
    }

    #[test]
    fn ensure_leading_slash_adds_missing() {
        assert_eq!(
            ensure_leading_slash("phrases/audio/x.opus"),
            "/phrases/audio/x.opus"
        );
    }

    #[test]
    fn ensure_leading_slash_preserves_existing() {
        assert_eq!(
            ensure_leading_slash("/phrases/audio/x.opus"),
            "/phrases/audio/x.opus"
        );
    }

    #[test]
    fn blob_url_cache_returns_none_for_uncached_key() {
        // Bug A contract: cache miss MUST surface as None to the caller so the
        // caller can fall back to TTS or await prefetch — it MUST NOT be silently
        // substituted with the raw CDN URL (see module-level root cause comment).
        let unique = "/blob-cache-probe-uncached-7e3a9f1c-e2d4-4b8a-9c11-1f5a2c7e9b8d";
        // Defensive cleanup in case a prior test populated this slot.
        if let Ok(mut cache) = blob_url_cache().lock() {
            cache.remove(unique);
        }
        let result = get_cached_blob_url(unique);
        assert!(
            result.is_none(),
            "uncached path must resolve to None; got {:?}",
            result
        );
    }

    #[test]
    fn blob_url_cache_round_trips_inserted_value() {
        let key = "/blob-cache-probe-roundtrip-3f8a1c0b-9d2e-4a6f-b117-2c4d8e1f0a3b";
        let value = "blob:roundtrip-test";
        {
            let mut cache = blob_url_cache().lock().expect("blob cache poisoned");
            cache.insert(key.to_string(), value.to_string());
        }
        assert_eq!(get_cached_blob_url(key).as_deref(), Some(value));
        // Cleanup so this test remains idempotent.
        if let Ok(mut cache) = blob_url_cache().lock() {
            cache.remove(key);
        }
    }

    #[test]
    fn mime_type_for_path_known_extensions() {
        assert_eq!(mime_type_for_path("phrases/audio/abc.opus"), "audio/opus");
        assert_eq!(mime_type_for_path("/x.mp3"), "audio/mpeg");
        assert_eq!(mime_type_for_path("/x.wav"), "audio/wav");
        assert_eq!(mime_type_for_path("/x.ogg"), "audio/ogg");
        assert_eq!(mime_type_for_path("/x.m4a"), "audio/mp4");
        assert_eq!(mime_type_for_path("/x.webm"), "audio/webm");
    }

    #[test]
    fn mime_type_for_path_unknown_defaults_to_opus() {
        // Current CDN only serves opus; the default keeps decode working for
        // the existing content while other extensions map explicitly above.
        assert_eq!(mime_type_for_path("noext"), "audio/opus");
        assert_eq!(mime_type_for_path("file.unknown"), "audio/opus");
    }

    #[test]
    fn blob_url_cache_insert_updates_existing_key_without_duplication() {
        let key = "/blob-cache-probe-update-1a2b3c4d-1111-2222-3333-444455556666";
        {
            let mut cache = blob_url_cache().lock().expect("blob cache poisoned");
            cache.insert(key.to_string(), "blob:first".to_string());
            // Re-inserting the same key must not append a duplicate order entry.
            cache.insert(key.to_string(), "blob:second".to_string());
            assert_eq!(cache.order.iter().filter(|k| *k == key).count(), 1);
        }
        assert_eq!(get_cached_blob_url(key).as_deref(), Some("blob:second"));
        if let Ok(mut cache) = blob_url_cache().lock() {
            cache.remove(key);
        }
    }
}
