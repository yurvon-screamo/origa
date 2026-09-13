//! WASM/browser test: the cache-first provider must refuse to touch the
//! network once the CDN was proven unreachable (ADR-052, slice 2).
//!
//! A cache miss under the unreachable flag must fail instantly with a
//! recognizable error — the offline startup budget (~3 s honest offline)
//! depends on no hanging requests.

#![cfg(all(target_arch = "wasm32", test))]

use wasm_bindgen_test::*;

use super::CdnUnreachableError;
use crate::repository::cdn_provider::{
    CacheFirstCdnProvider, FetchDecision, clear_cdn_unreachable, fetch_decision,
    is_cdn_unreachable, mark_cdn_unreachable,
};
use origa::traits::CdnProvider;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn cache_miss_fails_instantly_when_cdn_is_unreachable() {
    // Arrange: nothing cached under this unique path, the flag is set.
    clear_cdn_unreachable();
    let provider = CacheFirstCdnProvider;
    let path = "/offline-probe/never-cached.json";

    // Act + Assert: the refusal is immediate — no network round trip,
    // no idle deadline wait.
    let started = crate::utils::now_ms();
    let error = provider
        .fetch_text(path)
        .await
        .expect_err("unreachable CDN must fail a cache miss");
    let elapsed = crate::utils::now_ms() - started;

    assert!(CdnUnreachableError::is_match(&error), "got: {error:?}");
    assert!(
        elapsed < 900.0,
        "refusal must be immediate, took {elapsed} ms"
    );
}

#[wasm_bindgen_test]
async fn unreachable_flag_round_trips_and_clears() {
    clear_cdn_unreachable();
    assert!(!is_cdn_unreachable());

    crate::repository::cdn_provider::mark_cdn_unreachable();
    assert!(is_cdn_unreachable());

    clear_cdn_unreachable();
    assert!(!is_cdn_unreachable());
}

#[wasm_bindgen_test]
fn fetch_decision_matrix_prefers_cache_and_refuses_dead_network() {
    use FetchDecision::*;

    assert_eq!(fetch_decision(true, false), ServeFromCache);
    assert_eq!(fetch_decision(true, true), ServeFromCache);
    assert_eq!(fetch_decision(false, false), FetchFromNetwork);
    assert_eq!(fetch_decision(false, true), FailOffline);
}
