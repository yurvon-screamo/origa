#![cfg(all(target_arch = "wasm32", test))]

//! Browser integration tests for the dictionary-loader manifest verification
//! (task 20261107150931-k0c1rb7): `load_kanji`/`load_grammar`/`load_radicals`
//! fetch through `fetch_text_verified`, so a cache body that predates the
//! file's manifest tracking (valid JSON, stale content) heals via the
//! bypass-refetch instead of serving forever.
//!
//! The unit gate tests (`resolve_verified_text` with an injected provider,
//! cdn_provider.rs) cover the decision core; these tests cover the real glue
//! the units cannot reach: the process-global `REMOTE_MANIFEST` filled by the
//! legitimate `check_and_invalidate()` against a real manifest URL, the
//! browser Cache API as the cache-first source, and the production loader
//! call-site (`load_kanji`) end-to-end.
//!
//! Requires a CDN mirror at the compiled-in `ORIGA_CDN_BASE_URL` whose
//! manifest tracks the canonical dictionary paths (the CI wasm suite serves
//! the `cdn-data` artifact; locally serve a mirror with `manifest.json` plus
//! `dictionary/kanji.json`, `dictionary/radicals.json`,
//! `grammar/grammar_v3.json`, `counters/counters.json`).
//!
//! Run this subset locally:
//! `wasm-pack test --headless --chrome origa_ui --features csr -- verified_fetch_`
//! (name filter — the full suite additionally needs the dictionary corpus).

use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;

use crate::repository::cache_manager::{check_and_invalidate, remote_hash_for};
use crate::repository::cdn_provider::{
    CDN_CACHE_NAME, TextFreshness, cdn_cache_url, clear_cdn_unreachable, fetch_text_verified,
    mark_cdn_unreachable, text_freshness,
};

wasm_bindgen_test_configure!(run_in_browser);

const KANJI_PATH: &str = "dictionary/kanji.json";
const RADICALS_PATH: &str = "dictionary/radicals.json";
const GRAMMAR_PATH: &str = "grammar/grammar_v3.json";

/// Minimal valid kanji.json — `init_kanji` must accept it. Its content is
/// deliberately different from any real corpus, so its hash cannot collide
/// with a manifest hash of the real file.
const STALE_KANJI_JSON: &str = r#"{"kanji":[{"kanji":"曇","jlpt":"N1","used_in":1,"description_ru":["ТЕСТОВЫЙ УСТАРЕВШИЙ КОНТЕНТ"],"description_en":["stale test content"],"description_ko":["stale"],"description_vi":["stale"],"radicals":["日"],"popular_words":[],"on_readings":["どん"],"kun_readings":["くも"]}]}"#;

const STALE_RADICALS_JSON: &str =
    r#"{"radicals":{"一":{"strokes":1,"meaning":"stale test content"}}}"#;
const STALE_GRAMMAR_JSON: &str =
    r#"{"rules":[{"id":"stale-test-rule","title":"stale test content"}]}"#;

fn sha256_hex(text: &str) -> String {
    origa::dictionary::cdn_blob::sha256_bytes(text.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

async fn open_cache() -> web_sys::Cache {
    let window = web_sys::window().expect("no window");
    let caches = window.caches().expect("no CacheStorage");
    let cache = JsFuture::from(caches.open(CDN_CACHE_NAME))
        .await
        .expect("caches.open resolved");
    cache.dyn_into().expect("Cache")
}

async fn delete_cache_entry(cache: &web_sys::Cache, path: &str) {
    JsFuture::from(cache.delete_with_str(&cdn_cache_url(path)))
        .await
        .expect("cache.delete resolved");
}

async fn read_cache_text(cache: &web_sys::Cache, path: &str) -> Option<String> {
    let result = JsFuture::from(cache.match_with_str(&cdn_cache_url(path)))
        .await
        .expect("cache.match resolved");
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

/// Seeds a string-backed 200 response the same way the app's own writes land
/// in the cache (same realm — see kanji_art_heal_wasm_tests for the realm
/// quirk rationale).
async fn seed_cache_body(cache: &web_sys::Cache, path: &str, body: &str) {
    let response = web_sys::Response::new_with_opt_str(Some(body)).expect("Response");
    JsFuture::from(cache.put_with_str(&cdn_cache_url(path), &response))
        .await
        .expect("cache.put resolved");
}

/// Fetches the remote manifest through the legitimate phase entry point so
/// `remote_hash_for` answers exactly like on an app start.
async fn fetch_manifest_like_app_start() {
    check_and_invalidate()
        .await
        .expect("manifest phase must succeed against the mirror");
}

/// Deterministic offline guard: the unreachable verdict makes every network
/// leg refuse instantly (see kanji_art_heal_wasm_tests).
struct UnreachableGuard;
impl Drop for UnreachableGuard {
    fn drop(&mut self) {
        clear_cdn_unreachable();
    }
}

/// Main scenario, through the real loader call-site: a cache entry whose body
/// predates the file's manifest tracking (valid JSON, wrong content) must be
/// healed by `load_kanji` — the bypass-refetch overwrites the poisoned entry
/// with the manifest-fresh body.
#[wasm_bindgen_test]
async fn verified_fetch_stale_kanji_cache_heals_through_load_kanji() {
    let cache = open_cache().await;
    delete_cache_entry(&cache, KANJI_PATH).await;
    fetch_manifest_like_app_start().await;

    let manifest_hash = remote_hash_for(KANJI_PATH)
        .expect("mirror manifest must track dictionary/kanji.json for this test to be meaningful");

    seed_cache_body(&cache, KANJI_PATH, STALE_KANJI_JSON).await;
    assert_eq!(
        text_freshness(STALE_KANJI_JSON, Some(manifest_hash)),
        TextFreshness::Stale,
        "precondition: the seeded body must classify as stale against the manifest"
    );

    crate::loaders::data_loader::load_kanji()
        .await
        .expect("load_kanji must heal the stale cache body and load");

    let healed = read_cache_text(&cache, KANJI_PATH)
        .await
        .expect("cache entry must exist after the heal");
    assert_ne!(
        healed, STALE_KANJI_JSON,
        "the poisoned body must be overwritten by the bypass-refetch"
    );
    assert_eq!(
        sha256_hex(&healed),
        manifest_hash.to_ascii_lowercase(),
        "the cache must now hold the manifest-fresh body"
    );
    assert!(
        origa::dictionary::kanji::is_kanji_loaded(),
        "kanji data must be initialized after load_kanji"
    );
}

/// The other two migrated paths verify against the same manifest gate: stale
/// cache bodies for radicals and grammar are replaced by manifest-fresh ones.
#[wasm_bindgen_test]
async fn verified_fetch_radicals_and_grammar_paths_verify_against_manifest() {
    let cache = open_cache().await;
    delete_cache_entry(&cache, RADICALS_PATH).await;
    delete_cache_entry(&cache, GRAMMAR_PATH).await;
    fetch_manifest_like_app_start().await;

    for (path, stale_body) in [
        (RADICALS_PATH, STALE_RADICALS_JSON),
        (GRAMMAR_PATH, STALE_GRAMMAR_JSON),
    ] {
        let manifest_hash =
            remote_hash_for(path).unwrap_or_else(|| panic!("mirror manifest must track {path}"));

        seed_cache_body(&cache, path, stale_body).await;

        let served = fetch_text_verified(path)
            .await
            .unwrap_or_else(|e| panic!("verified fetch of {path} must succeed: {e:?}"));
        assert_ne!(
            served, stale_body,
            "stale body of {path} must not be served"
        );
        assert_eq!(
            sha256_hex(&served),
            manifest_hash.to_ascii_lowercase(),
            "{path} must resolve to the manifest-fresh body"
        );
    }
}

/// Fail-open contract: when the body diverges and the network is proven
/// unreachable, the refetch refuses instantly and the loader keeps the stale
/// body instead of turning a working stale resource into an error.
#[wasm_bindgen_test]
async fn verified_fetch_failed_refetch_keeps_stale_body_fail_open() {
    let cache = open_cache().await;
    fetch_manifest_like_app_start().await;

    let stale_body = r#"{"kanji":[],"stale":"offline-heal-must-keep-this-body"}"#;
    seed_cache_body(&cache, KANJI_PATH, stale_body).await;

    clear_cdn_unreachable();
    mark_cdn_unreachable();
    let _guard = UnreachableGuard;

    let served = fetch_text_verified(KANJI_PATH)
        .await
        .expect("fail-open must serve the stale body, not error");
    assert_eq!(
        served, stale_body,
        "an unreachable refetch must keep the cached stale body"
    );
}
