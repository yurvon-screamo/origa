#![cfg(all(target_arch = "wasm32", test))]

//! Behavior pin for #1486: the selftest listener must recover a deep link
//! that lands *after* it mounted, while the pushed `deep-link://new-url`
//! event is stalled in the WebView. The old one-shot cold poll could only
//! see a URL that was already pending at mount — a warm link arriving one
//! tick later was lost until the script's budget expired red.
//!
//! The Tauri bridge is scripted with [`TauriMock`]: there is no
//! `event.listen` (the push channel is dead), `get_current_deep_link`
//! reports an empty URL on the first tick and the selftest URL on the
//! second, and the OCR plugin reports the fixed glyphs. The assertion is on
//! the externally observable outcome — a `selftest_marker` invocation
//! carrying `[selftest] ocr ok`.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use js_sys::{Object, Reflect};
use leptos::prelude::*;
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;

use crate::core::selftest::start_selftest_listener;
use crate::core::tauri_mock::{TauriMock, UserAgentOverride};

wasm_bindgen_test_configure!(run_in_browser);

const CMD_CURRENT_DEEP_LINK: &str = "get_current_deep_link";
const CMD_OCR: &str = "plugin:device-ai-apis|vision_recognize_text";
const CMD_MARKER: &str = "selftest_marker";
const SELFTEST_URL: &str = "origa://selftest-ocr";

/// `spawn_local` tasks run on the leptos executor, which initialises on the
/// first mount (the same constraint as the other component tests).
fn init_leptos_executor() {
    let wrapper = crate::test_support::create_wrapper();
    crate::test_support::mount_to_wrapper(&wrapper, move || {
        view! { <div data-testid="selftest-poll-test-mount"></div> }.into_any()
    });
}

/// Scripted native bridge. `pull_urls` are returned by consecutive
/// `get_current_deep_link` calls; every marker invoke is recorded. The OCR
/// command always recognises the expected glyphs so the assertion is about
/// delivery, not about the recognition result.
struct ScriptedBridge {
    _mock: TauriMock,
    _ua: UserAgentOverride,
    pulls: Rc<Cell<usize>>,
    markers: Rc<RefCell<Vec<String>>>,
}

impl ScriptedBridge {
    fn install(pull_urls: &[&str]) -> Self {
        let pull_urls: Vec<String> = pull_urls.iter().map(|url| (*url).to_string()).collect();
        let pulls = Rc::new(Cell::new(0));
        let markers = Rc::new(RefCell::new(Vec::new()));
        let pulls_for_route = Rc::clone(&pulls);
        let markers_for_route = Rc::clone(&markers);

        let mock = TauriMock::install(move |command, args| {
            let command = command.as_string().unwrap_or_default();
            match command.as_str() {
                CMD_CURRENT_DEEP_LINK => {
                    let index = pulls_for_route.get();
                    pulls_for_route.set(index + 1);
                    pull_urls
                        .get(index)
                        .map(|url| JsValue::from_str(url))
                        .unwrap_or(JsValue::NULL)
                },
                CMD_OCR => {
                    let result = Object::new();
                    Reflect::set(
                        &result,
                        &JsValue::from_str("text"),
                        &JsValue::from_str("日本語"),
                    )
                    .expect("set OCR text");
                    result.into()
                },
                CMD_MARKER => {
                    let message = Reflect::get(&args, &JsValue::from_str("message"))
                        .ok()
                        .and_then(|value| value.as_string())
                        .unwrap_or_default();
                    markers_for_route.borrow_mut().push(message);
                    JsValue::UNDEFINED
                },
                // No `event.listen` entry: the push channel is deliberately
                // absent, so only the pull path can deliver the URL.
                _ => JsValue::UNDEFINED,
            }
        });
        // `platform_name()` gates the listener on Android; the scripted UA
        // reproduces the smoke contour inside a plain browser page.
        let ua = UserAgentOverride::install("Mozilla/5.0 (Android 14; Mobile)");
        Self {
            _mock: mock,
            _ua: ua,
            pulls,
            markers,
        }
    }

    async fn wait_for_marker(&self, attempts: usize) -> Vec<String> {
        crate::test_support::wait_until(|| !self.markers.borrow().is_empty(), attempts, 100).await;
        self.markers.borrow().clone()
    }
}

#[wasm_bindgen_test]
async fn selftest_link_arriving_after_mount_is_recovered_by_the_pull_poll() {
    init_leptos_executor();
    let bridge = ScriptedBridge::install(&["", SELFTEST_URL]);

    start_selftest_listener();
    let markers = bridge.wait_for_marker(70).await;

    assert!(
        bridge.pulls.get() >= 2,
        "a one-shot cold poll cannot see a link that lands after mount"
    );
    assert!(
        markers.iter().any(|m| m.contains("[selftest] ocr ok")),
        "the late selftest link must reach OCR through the pull path; got {markers:?}"
    );
}

#[wasm_bindgen_test]
async fn polled_foreign_deep_link_is_never_routed_to_the_selftest_probe() {
    init_leptos_executor();
    let bridge = ScriptedBridge::install(&["origa://auth/callback?code=x"]);

    start_selftest_listener();
    // Two tick windows: a foreign URL must not trigger OCR on any tick, and
    // the OAuth contour that shares this channel must stay untouched.
    gloo_timers::future::TimeoutFuture::new(3500).await;

    assert!(
        bridge.pulls.get() >= 2,
        "the poll must keep ticking across the warm-delivery window"
    );
    assert!(
        bridge.markers.borrow().is_empty(),
        "only selftest URLs may be routed; got {:?}",
        bridge.markers
    );
}
