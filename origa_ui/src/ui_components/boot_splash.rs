//! Static boot splash lifecycle.
//!
//! The splash is static HTML in `index.html` (visible before WASM boots),
//! preventing an empty cream background during cold start. Once `App` mounts
//! and the first frame is rendered, this fades the splash out and removes it.
//!
//! The CSS transition duration in `index.html` (`#origa-boot-splash`) MUST stay
//! in sync with `FADE_OUT_MS` here — they are the two ends of the same fade.

use gloo_timers::future::TimeoutFuture;
use leptos::task::spawn_local;
use leptos::wasm_bindgen::JsCast;
use leptos::wasm_bindgen::prelude::Closure;
use std::sync::atomic::{AtomicBool, Ordering};

const SPLASH_ID: &str = "origa-boot-splash";
const HIDDEN_CLASS: &str = "origa-boot-splash--hidden";
/// Delay before fading: lets Leptos commit the first view frame to the DOM so
/// the cream background does not flash through during the fade.
const SETTLE_MS: u32 = 60;
/// Must match the `transition: opacity` duration on `#origa-boot-splash` in
/// `index.html`. The node is removed after this window.
const FADE_OUT_MS: u32 = 300;

static HIDE_STARTED: AtomicBool = AtomicBool::new(false);

/// Hide the boot splash once the app has mounted.
///
/// Guarded by a once-flag: even if called multiple times during the settle
/// window, only the first call proceeds (no duplicate timers, no double
/// `remove()`). Safe to call from re-mounts / tests after the splash is gone.
pub fn hide_boot_splash() {
    if HIDE_STARTED.swap(true, Ordering::Relaxed) {
        return;
    }

    spawn_local(async move {
        TimeoutFuture::new(SETTLE_MS).await;

        let Some(document) = web_sys::window().and_then(|w| w.document()) else {
            return;
        };
        let Some(splash) = document.get_element_by_id(SPLASH_ID) else {
            return;
        };

        let _ = splash.class_list().add_1(HIDDEN_CLASS);

        let removal_target = splash.clone();
        let removal = Closure::<dyn FnMut()>::new(move || {
            removal_target.remove();
        });
        let registered = web_sys::window().is_some_and(|w| {
            w.set_timeout_with_callback_and_timeout_and_arguments(
                removal.as_ref().unchecked_ref(),
                FADE_OUT_MS as i32,
                &js_sys::Array::new(),
            )
            .is_ok()
        });
        if registered {
            removal.forget();
        } else {
            // setTimeout unavailable (extreme edge): remove synchronously so the
            // node never lingers as an invisible overlay capturing nothing but
            // still sitting in the DOM.
            drop(removal);
            splash.remove();
        }
    });
}
