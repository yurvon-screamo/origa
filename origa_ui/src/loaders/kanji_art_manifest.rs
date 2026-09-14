//! Kanji art availability manifest (#540): the offline-generated list of
//! kanji that actually have `kanji_animations/<kanji>.svg` /
//! `kanji_frames/<kanji>.svg` objects on the CDN.
//!
//! Two complementary roles (see ADR-048-adjacent #540 plan):
//! - **Primary requests** (the card pre-cache builds its download list
//!   before any runtime lookups happen): `precache_all_cards` awaits
//!   [`ensure_kanji_art_manifest`] first, then `kanji_svg_resources`
//!   filters paths against the manifest — kanji without art produce no
//!   request at all.
//! - **Runtime repeats** (`KanjiAnimation` re-mounts on hover): the
//!   session-level negative cache records confirmed misses so a kanji
//!   without art is not re-fetched on every mount.
//!
//! A missing/unreadable manifest (offline first run, old CDN) degrades to
//! the pre-manifest behavior: no filtering, negative cache only.

use std::collections::HashSet;
use std::sync::Mutex;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU8, Ordering};

use origa::domain::OrigaError;
use origa::traits::CdnProvider;
use serde::Deserialize;

use crate::loaders::kanji_bundle_store::KanjiBundleType;

pub const KANJI_ART_MANIFEST_PATH: &str = "kanji_art_manifest.json";

/// Lifecycle of the manifest: 0 = idle, 1 = loading, 2 = ready,
/// 3 = unavailable (next caller retries).
static MANIFEST_STATE: AtomicU8 = AtomicU8::new(0);
const MANIFEST_IDLE: u8 = 0;
const MANIFEST_LOADING: u8 = 1;
const MANIFEST_READY: u8 = 2;
const MANIFEST_UNAVAILABLE: u8 = 3;

/// `RwLock<Option<…>>` (not `OnceLock`) so tests can reset the process
/// global between cases; production installs exactly once.
static MANIFEST: std::sync::RwLock<Option<KanjiArtManifest>> = std::sync::RwLock::new(None);

/// Session-level negative cache: art paths confirmed missing at runtime.
static NEGATIVE_CACHE: OnceLock<Mutex<HashSet<(KanjiBundleType, char)>>> = OnceLock::new();

fn negative_cache() -> &'static Mutex<HashSet<(KanjiBundleType, char)>> {
    NEGATIVE_CACHE.get_or_init(|| Mutex::new(HashSet::new()))
}

#[derive(Debug, Default, Deserialize)]
pub struct KanjiArtManifest {
    pub frames: HashSet<char>,
    pub animations: HashSet<char>,
}

impl KanjiArtManifest {
    fn contains(&self, bundle_type: KanjiBundleType, kanji: char) -> bool {
        match bundle_type {
            KanjiBundleType::Animations => self.animations.contains(&kanji),
            KanjiBundleType::Frames => self.frames.contains(&kanji),
        }
    }

    pub fn from_json(json: &str) -> Result<Self, OrigaError> {
        serde_json::from_str(json).map_err(|e| OrigaError::RepositoryError {
            reason: format!("{KANJI_ART_MANIFEST_PATH} is unreadable: {e}"),
        })
    }
}

/// Ensures the manifest is loaded, fetching it on demand. Concurrent
/// callers coalesce onto the in-flight fetch (yield-loop wait); a caller
/// arriving after a failed fetch retries it. Failure is non-fatal — the
/// callers keep the unfiltered behavior.
pub async fn ensure_kanji_art_manifest() -> Result<(), OrigaError> {
    if MANIFEST_STATE.load(Ordering::Acquire) == MANIFEST_READY {
        return Ok(());
    }

    let took_over = MANIFEST_STATE
        .compare_exchange(
            MANIFEST_IDLE,
            MANIFEST_LOADING,
            Ordering::AcqRel,
            Ordering::Acquire,
        )
        .is_ok()
        || MANIFEST_STATE
            .compare_exchange(
                MANIFEST_UNAVAILABLE,
                MANIFEST_LOADING,
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .is_ok();
    if took_over {
        let result = load_kanji_art_manifest_via(crate::repository::cdn_provider()).await;
        // `load_via` flips the state to READY on success; failures are
        // marked here so waiting callers see UNAVAILABLE (the next
        // caller retries).
        if result.is_err() {
            MANIFEST_STATE.store(MANIFEST_UNAVAILABLE, Ordering::Release);
            tracing::warn!("Kanji art manifest unavailable, art filtering disabled: {result:?}");
        }
        return result;
    }

    // Another task is loading: wait for it to conclude (its failure
    // surfaces here; the next caller retries).
    loop {
        crate::utils::yield_to_browser().await;
        match MANIFEST_STATE.load(Ordering::Acquire) {
            MANIFEST_READY => return Ok(()),
            MANIFEST_UNAVAILABLE => {
                return Err(OrigaError::RepositoryError {
                    reason: "kanji art manifest failed to load".to_string(),
                });
            },
            MANIFEST_IDLE => return Box::pin(ensure_kanji_art_manifest()).await,
            _ => {},
        }
    }
}

/// Provider-parameterized core: fetch + parse + install (the install
/// flips the lifecycle state to READY — waiting `ensure` callers observe
/// it through the same atomic). Testable with a mock provider.
pub async fn load_kanji_art_manifest_via<P: CdnProvider>(provider: &P) -> Result<(), OrigaError> {
    let json = provider.fetch_text(KANJI_ART_MANIFEST_PATH).await?;
    let manifest = KanjiArtManifest::from_json(&json)?;
    tracing::info!(
        frames = manifest.frames.len(),
        animations = manifest.animations.len(),
        "Kanji art manifest loaded"
    );
    *MANIFEST.write().map_err(|e| OrigaError::RepositoryError {
        reason: format!("kanji art manifest lock: {e:?}"),
    })? = Some(manifest);
    MANIFEST_STATE.store(MANIFEST_READY, Ordering::Release);
    Ok(())
}

/// Art availability for one kanji. `None` = manifest not ready — callers
/// must NOT filter in that case (graceful pre-manifest behavior).
pub fn kanji_art_exists(bundle_type: KanjiBundleType, kanji: char) -> Option<bool> {
    if MANIFEST_STATE.load(Ordering::Acquire) != MANIFEST_READY {
        return None;
    }
    MANIFEST
        .read()
        .ok()
        .and_then(|guard| guard.as_ref().map(|m| m.contains(bundle_type, kanji)))
}

/// Records a runtime-confirmed miss so repeated mounts of the same kanji
/// skip the doomed fetch for the rest of the session.
pub fn record_kanji_art_miss(bundle_type: KanjiBundleType, kanji: char) {
    negative_cache()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .insert((bundle_type, kanji));
}

/// Whether a miss was already confirmed at runtime this session.
pub fn is_known_kanji_art_miss(bundle_type: KanjiBundleType, kanji: char) -> bool {
    negative_cache()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .contains(&(bundle_type, kanji))
}

/// Drops the installed manifest and the negative cache. Test isolation
/// only — production never resets (the manifest is static for the
/// process lifetime).
#[cfg(test)]
pub(crate) fn reset_kanji_art_state() {
    MANIFEST_STATE.store(MANIFEST_IDLE, Ordering::Release);
    *MANIFEST.write().unwrap_or_else(|e| e.into_inner()) = None;
    negative_cache()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clear();
}

/// Test-only direct install (no CDN round-trip) for consumers' unit
/// tests, e.g. `kanji_svg_resources` filtering.
#[cfg(test)]
pub(crate) fn install_manifest_for_test(manifest: KanjiArtManifest) {
    *MANIFEST.write().unwrap_or_else(|e| e.into_inner()) = Some(manifest);
    MANIFEST_STATE.store(MANIFEST_READY, Ordering::Release);
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::future::Future;
    use std::sync::Mutex;

    use super::*;

    /// Serializes tests that mutate the process-global manifest state —
    /// parallel install/reset of the same store is a race otherwise.
    static STATE_TEST_LOCK: Mutex<()> = Mutex::new(());

    /// Recording mock over the external CDN boundary (same pattern as
    /// grammar_precompute_loader tests).
    struct MockCdn {
        manifest_json: RefCell<Option<String>>,
        fetches: RefCell<Vec<String>>,
    }

    impl MockCdn {
        fn with(manifest_json: Option<&str>) -> Self {
            Self {
                manifest_json: RefCell::new(manifest_json.map(String::from)),
                fetches: RefCell::new(Vec::new()),
            }
        }
    }

    impl CdnProvider for MockCdn {
        fn fetch_text(&self, path: &str) -> impl Future<Output = Result<String, OrigaError>> {
            self.fetches.borrow_mut().push(path.to_string());
            let result = match (path, self.manifest_json.borrow_mut().take()) {
                (KANJI_ART_MANIFEST_PATH, Some(json)) => Ok(json),
                _ => Err(OrigaError::NetworkError {
                    url: path.to_string(),
                    reason: "not stubbed".to_string(),
                }),
            };
            std::future::ready(result)
        }

        fn fetch_bytes(&self, path: &str) -> impl Future<Output = Result<Vec<u8>, OrigaError>> {
            self.fetches.borrow_mut().push(path.to_string());
            std::future::ready(Err(OrigaError::NetworkError {
                url: path.to_string(),
                reason: "not stubbed".to_string(),
            }))
        }
    }

    fn sample_manifest_json() -> String {
        r#"{"frames": ["日", "本"], "animations": ["日"]}"#.to_string()
    }

    #[tokio::test]
    #[expect(
        clippy::await_holding_lock,
        reason = "the std lock serializes tests over the process-global store; the awaited load runs on the test's single-threaded runtime, so no cross-task deadlock is possible"
    )]
    async fn loaded_manifest_answers_availability_lookups() {
        let _guard = STATE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        // Arrange
        reset_kanji_art_state();
        let provider = MockCdn::with(Some(&sample_manifest_json()));

        // Act
        load_kanji_art_manifest_via(&provider).await.unwrap();

        // Assert
        assert_eq!(
            kanji_art_exists(KanjiBundleType::Frames, '日'),
            Some(true),
            "manifest-listed kanji exists"
        );
        assert_eq!(
            kanji_art_exists(KanjiBundleType::Animations, '本'),
            Some(false),
            "per-kind sets are independent"
        );
        assert_eq!(
            kanji_art_exists(KanjiBundleType::Frames, '語'),
            Some(false),
            "unlisted kanji is a miss"
        );
        reset_kanji_art_state();
    }

    #[tokio::test]
    async fn lookups_without_manifest_do_not_filter() {
        let _guard = STATE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        // Arrange: nothing loaded.
        reset_kanji_art_state();

        // Act / Assert: None = "unknown", callers must not filter.
        assert_eq!(kanji_art_exists(KanjiBundleType::Frames, '日'), None);
    }

    #[tokio::test]
    async fn broken_manifest_json_is_rejected() {
        // Arrange
        let provider = MockCdn::with(Some("not json"));

        // Act
        let result = load_kanji_art_manifest_via(&provider).await;

        // Assert
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn negative_cache_suppresses_repeat_lookups_within_session() {
        let _guard = STATE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        // Arrange
        reset_kanji_art_state();
        assert!(!is_known_kanji_art_miss(KanjiBundleType::Frames, '意'));

        // Act
        record_kanji_art_miss(KanjiBundleType::Frames, '意');

        // Assert
        assert!(is_known_kanji_art_miss(KanjiBundleType::Frames, '意'));
        assert!(!is_known_kanji_art_miss(KanjiBundleType::Animations, '意'));
        reset_kanji_art_state();
    }

    #[tokio::test]
    #[expect(
        clippy::await_holding_lock,
        reason = "see loaded_manifest_answers_availability_lookups"
    )]
    async fn reset_clears_the_installed_state() {
        let _guard = STATE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        // Arrange
        reset_kanji_art_state();
        let provider = MockCdn::with(Some(&sample_manifest_json()));
        load_kanji_art_manifest_via(&provider).await.unwrap();
        assert_eq!(kanji_art_exists(KanjiBundleType::Frames, '日'), Some(true));

        // Act
        reset_kanji_art_state();

        // Assert: back to the unfiltered (manifest-unknown) behavior —
        // the entry point for a fresh install is `ensure_…` again.
        assert_eq!(kanji_art_exists(KanjiBundleType::Frames, '日'), None);
    }
}
