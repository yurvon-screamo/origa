#![cfg(all(target_arch = "wasm32", test))]

//! Browser integration test for the kanji-art cache self-heal: a Cache
//! API entry poisoned with a garbage 200 body (the 2026-09-19 CDN
//! migration-window failure) must be recognized as implausible, purged
//! and treated as a retryable miss.
//!
//! The poison is seeded through `web_sys::Response::new_with_opt_str` —
//! the same realm as the reader, exactly like `store_cache_marker` does
//! it. (A `Response` created from a Playwright evaluate context hangs
//! the WASM `cache.match`/`text()` reads — a browser-level quirk, which
//! is why this regression lives here and not in the Playwright suite;
//! the heal itself is realm-agnostic, only the seeding differs.)
//!
//! Deterministic offline mode: the unreachable verdict makes every
//! network leg refuse instantly, so the test exercises the real glue —
//! cache match, plausibility detection, purge, bypass-refetch
//! classification — with zero network dependency. The successful-refetch
//! half of the heal runs against the real CDN in local development
//! (same test body, flag cleared).

use leptos::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;

use crate::loaders::kanji_art_manifest::{fetch_kanji_art_svg, reset_kanji_art_state};
use crate::loaders::kanji_bundle_store::KanjiBundleType;
use crate::repository::cdn_provider::{
    CDN_CACHE_NAME, cdn_cache_url, clear_cdn_unreachable, mark_cdn_unreachable,
};

wasm_bindgen_test_configure!(run_in_browser);

const KANJI: &str = "医";
const ART_PATH: &str = "kanji_animations/%E5%8C%BB.svg";
const POISON: &str = "<html>poisoned art body</html>";

/// `fetch_kanji_art_svg` дергает `leptos::task::spawn_local` (detached
/// загрузчик), а глобальный Leptos-исполнитель инициализируется
/// монтированием. Тест ничего не монтирует — зависимость от порядка
/// параллельных тестов в документе деградировала во флак: монтируем
/// пустой компонент явно, исполнитель инициализируется на глазах у
/// рантайма, spawn_local в проверяемом пути становится легальным.
fn init_leptos_executor() {
    let wrapper = crate::test_support::create_wrapper();
    crate::test_support::mount_to_wrapper(&wrapper, move || {
        view! { <div data-testid="kanji-art-test-mount"></div> }.into_any()
    });
}

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
async fn poisoned_kanji_art_entry_is_detected_and_purged() {
    init_leptos_executor();
    reset_kanji_art_state();
    let cache = open_cache().await;

    // Deterministic offline mode: every network leg refuses instantly
    // (the unreachable verdict), so the test exercises the real glue —
    // cache match, plausibility detection, purge, bypass-refetch
    // classification — with zero network dependency.
    struct UnreachableGuard;
    impl Drop for UnreachableGuard {
        fn drop(&mut self) {
            clear_cdn_unreachable();
        }
    }
    clear_cdn_unreachable();
    mark_cdn_unreachable();
    let _guard = UnreachableGuard;

    // Seed the poison the way the app's own markers are written (same
    // realm, string-backed 200) — the plausibility check is the only
    // thing that can tell it apart from real art.
    let poison_response = web_sys::Response::new_with_opt_str(Some(POISON)).expect("Response");
    JsFuture::from(cache.put_with_str(&cdn_cache_url(ART_PATH), &poison_response))
        .await
        .expect("cache.put resolved");

    // The shared art path: cache hit (poison) → implausible → purge →
    // bypass refetch refused (offline) → Offline (retryable, no
    // negative caching).
    let healed = fetch_kanji_art_svg(KanjiBundleType::Animations, KANJI, ART_PATH).await;
    assert!(healed.is_none(), "an unreachable CDN must not produce art");

    // THE core guarantee: the poison does not survive the read — the
    // purge removed it even though the refetch failed offline.
    let cached = read_cache_text(&cache, ART_PATH).await;
    assert!(
        cached.is_none(),
        "the poisoned entry must be purged, got: {:?}",
        cached.as_deref().map(|text| &text[..text.len().min(60)])
    );

    reset_kanji_art_state();
}
