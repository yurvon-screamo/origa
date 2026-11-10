#![cfg(all(target_arch = "wasm32", test))]

//! Browser test for the `KanjiDrawingPractice` stroke-progress label
//! placement contract: the label and the canvas share a canvas-sized
//! positioning context (`.kanji-drawing-stage`), so the label stays
//! pinned to the drawing box in every parent layout (the kanji-detail
//! practice cell and the lesson writing card) instead of hugging a
//! full-width container edge, where it could leave the viewport.
//!
//! The label only renders in the non-error branch, which needs the
//! kanji art to resolve. Deterministic offline setup (the
//! `kanji_art_heal_wasm_tests` recipe): a plausible SVG is seeded into
//! the CDN Cache Storage (same realm, string-backed 200) and the CDN is
//! marked unreachable — cache-first hits the seed, every network leg
//! refuses instantly, the manifest gate degrades to unfiltered.

use leptos::prelude::*;
use leptos::task::tick;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;

use crate::loaders::kanji_art_manifest::reset_kanji_art_state;
use crate::repository::cdn_provider::{
    CDN_CACHE_NAME, cdn_cache_url, clear_cdn_unreachable, mark_cdn_unreachable,
};
use crate::ui_components::KanjiDrawingPractice;

wasm_bindgen_test_configure!(run_in_browser);

/// 漢 — unused by the other kanji-art tests, so parallel cache access in
/// the same browser document cannot race on this entry.
const KANJI: &str = "漢";
const SVG: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\"><path d=\"M 20 20 L 80 80\" fill=\"none\"/></svg>";

struct UnreachableGuard;
impl Drop for UnreachableGuard {
    fn drop(&mut self) {
        clear_cdn_unreachable();
    }
}

async fn open_cache() -> web_sys::Cache {
    let window = web_sys::window().expect("no window");
    let caches = window.caches().expect("no CacheStorage");
    let cache = JsFuture::from(caches.open(CDN_CACHE_NAME))
        .await
        .expect("caches.open resolved");
    cache.dyn_into().expect("Cache")
}

async fn seed_kanji_art(cache: &web_sys::Cache, path: &str) {
    let response = web_sys::Response::new_with_opt_str(Some(SVG)).expect("Response");
    JsFuture::from(cache.put_with_str(&cdn_cache_url(path), &response))
        .await
        .expect("cache.put resolved");
}

async fn purge_kanji_art(cache: &web_sys::Cache, path: &str) {
    JsFuture::from(cache.delete_with_str(&cdn_cache_url(path)))
        .await
        .expect("cache.delete resolved");
}

/// Polls until the predicate holds, bounded: the LocalResource and the
/// effects settle within a few browser yields; the bound only guards
/// against a hung fetch (which the unreachable verdict prevents).
async fn wait_for(predicate: impl Fn() -> bool) -> bool {
    for _ in 0..50 {
        if predicate() {
            return true;
        }
        crate::utils::yield_to_browser().await;
        tick().await;
    }
    predicate()
}

#[wasm_bindgen_test]
async fn stroke_progress_label_shares_the_canvas_positioning_context() {
    reset_kanji_art_state();
    clear_cdn_unreachable();
    mark_cdn_unreachable();
    let _guard = UnreachableGuard;

    let cache = open_cache().await;
    let art_path = format!("kanji_animations/{}.svg", urlencoding::encode(KANJI));
    seed_kanji_art(&cache, &art_path).await;

    // `fetch_kanji_art_svg` reaches `spawn_local` (the detached manifest
    // loader), which needs the Leptos executor — initialized by a mount
    // (the kanji_art_heal_wasm_tests recipe). This probe mount stays up
    // for the whole test.
    let executor_wrapper = crate::test_support::create_wrapper();
    crate::test_support::mount_to_wrapper(&executor_wrapper, move || {
        view! { <div data-testid="kanji-drawing-test-executor"></div> }.into_any()
    });

    // Isolation first: the seeded entry must resolve through the same
    // shared fetch the component uses — a failure here is a test-setup
    // problem, not a component problem.
    let direct =
        crate::loaders::kanji_art_manifest::fetch_kanji_art_svg(
            crate::loaders::kanji_bundle_store::KanjiBundleType::Animations,
            KANJI,
            &art_path,
        )
        .await;
    assert!(
        direct.is_some(),
        "the seeded art must resolve through fetch_kanji_art_svg in offline mode"
    );

    let wrapper = crate::test_support::create_wrapper();
    crate::test_support::mount_with_i18n(&wrapper, move || {
        view! {
            <KanjiDrawingPractice kanji={KANJI.to_string()} test_id="drawing" />
        }
        .into_any()
    });

    // Act + Assert: once the art resolves, the progress label must live
    // INSIDE the canvas stage — one positioning context for the label
    // and the canvas, so the corner pin anchors to the drawing box.
    let label_in_stage = wait_for(|| {
        wrapper
            .query_selector(".kanji-drawing-stage [data-testid=\"drawing-progress\"]")
            .is_ok_and(|el| el.is_some())
    })
    .await;
    assert!(
        label_in_stage,
        "the stroke-progress label must render inside the canvas stage; DOM: {}",
        wrapper.inner_html()
    );

    let stage_wraps_canvas = wrapper
        .query_selector(".kanji-drawing-stage .kanji-drawing-canvas")
        .is_ok_and(|el| el.is_some());
    assert!(
        stage_wraps_canvas,
        "the stage must wrap the canvas; DOM: {}",
        wrapper.inner_html()
    );

    let label_text = wait_for(|| {
        let node = wrapper
            .query_selector(".kanji-drawing-stage [data-testid=\"drawing-progress\"]");
        let Some(label) = node.ok().flatten() else {
            return false;
        };
        label
            .text_content()
            .map(|text| text.contains("1 / 1"))
            .unwrap_or(false)
    })
    .await;
    assert!(
        label_text,
        "the label must show the stroke progress (1 / 1); DOM: {}",
        wrapper.inner_html()
    );

    purge_kanji_art(&cache, &art_path).await;
    reset_kanji_art_state();
}
