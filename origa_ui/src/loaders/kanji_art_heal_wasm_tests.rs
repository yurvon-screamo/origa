#![cfg(all(target_arch = "wasm32", test))]

//! Browser integration test for the kanji-art cache self-heal: a Cache
//! API entry poisoned with a garbage 200 body (the 2026-09-19 CDN
//! migration-window failure) must be recognized as implausible, purged
//! and refetched from the network.
//!
//! The poison is seeded through `web_sys::Response::new_with_opt_str` —
//! the same realm as the reader, exactly like `store_cache_marker` does
//! it. (A `Response` created from a Playwright evaluate context hangs
//! the WASM `cache.match`/`text()` reads — a browser-level quirk, which
//! is why this regression lives here and not in the Playwright suite;
//! the heal itself is realm-agnostic, only the seeding differs.)
//!
//! The healed refetch downloads the real `kanji_animations/医.svg` from
//! the compiled-in production CDN base, so the test is a true
//! integration of the provider + Cache API + plausibility gates.

use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;

use crate::loaders::kanji_art_manifest::{fetch_kanji_art_svg, reset_kanji_art_state};
use crate::loaders::kanji_bundle_store::KanjiBundleType;
use crate::repository::cdn_provider::{CDN_CACHE_NAME, cdn_cache_url};
use origa::traits::CdnProvider;

wasm_bindgen_test_configure!(run_in_browser);

const KANJI: &str = "医";
const ART_PATH: &str = "kanji_animations/%E5%8C%BB.svg";
const POISON: &str = "<html>poisoned art body</html>";

async fn open_cache() -> web_sys::Cache {
    let window = web_sys::window().expect("no window");
    let caches = window.caches().expect("no CacheStorage");
    let cache = JsFuture::from(caches.open(CDN_CACHE_NAME))
        .await
        .expect("caches.open resolved");
    cache.dyn_into().expect("Cache")
}

async fn read_cache_text(cache: &web_sys::Cache, path: &str) -> Option<String> {
    let result = JsFuture::from(cache.match_with_str(&cdn_cache_url(path)))
        .await
        .ok()?;
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

#[wasm_bindgen_test]
async fn poisoned_kanji_art_entry_is_purged_and_refetched() {
    reset_kanji_art_state();
    let cache = open_cache().await;

    // Seed the poison the way the app's own markers are written (same
    // realm, string-backed 200) — the plausibility check is the only
    // thing that can tell it apart from real art.
    let poison_response = web_sys::Response::new_with_opt_str(Some(POISON)).expect("Response");
    JsFuture::from(cache.put_with_str(&cdn_cache_url(ART_PATH), &poison_response))
        .await
        .expect("cache.put resolved");

    // The shared art path: cache hit (poison) → purge → network refetch.
    let healed = fetch_kanji_art_svg(KanjiBundleType::Animations, KANJI, ART_PATH).await;

    // THE core guarantee, provable in every environment: the poison does
    // not survive the read. Either the purged entry was overwritten with
    // the healed body, or the refetch failed and the entry is gone —
    // both leave the cache poison-free.
    let cached = read_cache_text(&cache, ART_PATH).await;
    assert!(
        cached
            .as_deref()
            .map_or(true, |text| !text.contains("poisoned")),
        "the poisoned entry must not survive the read, got: {:?}",
        cached.as_deref().map(|text| &text[..text.len().min(60)])
    );

    match healed {
        Some(svg) => {
            assert!(
                svg.starts_with("<svg"),
                "healed body must be an SVG, got: {:?}",
                &svg[..svg.len().min(60)]
            );
            let cached = cached.expect("the healed body must be cached");
            assert!(cached.starts_with("<svg"));
        },
        None => {
            // The refetch failed: acceptable only when this environment
            // cannot reach the production CDN at all (CI runners vs the
            // RU-hosted origin). If a direct fetch succeeds, the heal
            // must have succeeded too — a None here is a regression.
            let direct = crate::repository::cdn_provider().fetch_text(ART_PATH).await;
            assert!(
                direct.is_err(),
                "the heal returned None while the CDN is reachable — regression"
            );
            web_sys::console::warn_1(
                &format!(
                    "production CDN unreachable in this environment, asserting purge only: {direct:?}"
                )
                .into(),
            );
        },
    }

    reset_kanji_art_state();
}
