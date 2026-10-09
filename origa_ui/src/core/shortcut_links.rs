//! Deep-link routing for `origa://add?source=…` (L-1 app shortcuts).
//!
//! Shortcut targets (`origa://add?source=camera|audio|file|text`) navigate
//! to /words and open the add-words drawer on the matching tab. The
//! routing piggybacks the existing deep-link delivery (event + cold-start
//! poll) — the oauth listener owns `origa://auth/*`, this module owns
//! `origa://add*`.

use tracing::debug;

/// The tab a shortcut opens the drawer on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShortcutSource {
    Camera,
    Audio,
    File,
    Text,
}

impl ShortcutSource {
    pub fn tab_id(&self) -> &'static str {
        match self {
            ShortcutSource::Camera => "image",
            ShortcutSource::Audio => "audio",
            ShortcutSource::File => "image", // file → same tab as camera (image picker)
            ShortcutSource::Text => "text",
        }
    }
}

/// Parses `origa://add?source=X` into a shortcut source. Returns None for
/// non-add URLs (oauth callbacks etc.) and unknown sources.
pub fn parse_shortcut_url(url: &str) -> Option<ShortcutSource> {
    if !url.starts_with("origa://add") {
        return None;
    }
    let source = url
        .split("source=")
        .nth(1)
        .map(|s| s.split('&').next().unwrap_or(s))?;
    match source {
        "camera" => Some(ShortcutSource::Camera),
        "audio" => Some(ShortcutSource::Audio),
        "file" => Some(ShortcutSource::File),
        "text" => Some(ShortcutSource::Text),
        _ => {
            debug!(source, "unknown shortcut source");
            None
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest::rstest]
    #[case::camera("origa://add?source=camera", Some(ShortcutSource::Camera))]
    #[case::audio("origa://add?source=audio", Some(ShortcutSource::Audio))]
    #[case::file("origa://add?source=file", Some(ShortcutSource::File))]
    #[case::text("origa://add?source=text", Some(ShortcutSource::Text))]
    #[case::oauth_url("origa://auth/callback?code=x", None)]
    #[case::bare_add("origa://add", None)]
    #[case::unknown_source("origa://add?source=qr", None)]
    #[case::extra_params("origa://add?source=audio&foo=bar", Some(ShortcutSource::Audio))]
    fn shortcut_urls_parse(#[case] url: &str, #[case] expected: Option<ShortcutSource>) {
        assert_eq!(parse_shortcut_url(url), expected);
    }
}

use wasm_bindgen::JsCast;

// Global take-once parking for a pending shortcut target.
thread_local! {
    static PENDING_TAB: std::cell::RefCell<Option<&'static str>> =
        const { std::cell::RefCell::new(None) };
}

/// Parks a shortcut target tab for the Words page to consume.
pub fn park_shortcut_tab(tab_id: &'static str) {
    PENDING_TAB.with(|slot| *slot.borrow_mut() = Some(tab_id));
}

/// Takes the parked shortcut tab.
pub fn take_shortcut_tab() -> Option<&'static str> {
    PENDING_TAB.with(|slot| slot.borrow_mut().take())
}

/// Whether a shortcut tab is parked (peek — the nav cycle checks without
/// consuming; the modal's mount consumes).
pub fn has_shortcut_tab() -> bool {
    PENDING_TAB.with(|slot| slot.borrow().is_some())
}

/// Starts listening for `origa://add` deep links (event + cold-start
/// poll). Coexists with the share-intake listener on the same channel.
pub fn start_shortcut_listener() {
    use crate::core::tauri::{event_listen_fn, invoke_with_args, is_tauri};
    use wasm_bindgen::JsValue;

    if !is_tauri() {
        return;
    }

    // Cold start: the deep-link plugin may hold the URL before the
    // listener mounts.
    spawn_local_or_ignore(async move {
        match invoke_with_args("get_current_deep_link", &JsValue::UNDEFINED).await {
            Ok(url) => {
                if let Some(url) = url.as_string() {
                    route_shortcut(&url);
                }
            },
            Err(e) => debug!("shortcut: cold-poll failed: {e}"),
        }
    });

    let Some(listen) = event_listen_fn() else {
        return;
    };
    let callback = wasm_bindgen::closure::Closure::wrap(Box::new(move |event: JsValue| {
        let url = js_sys::Reflect::get(&event, &JsValue::from_str("url"))
            .ok()
            .and_then(|v| v.as_string());
        if let Some(url) = url {
            route_shortcut(&url);
        }
    }) as Box<dyn Fn(JsValue)>);
    if let Some(callback_ptr) = callback.as_ref().dyn_ref::<js_sys::Function>() {
        // listen(event_name, handler, options?) — the share-intake pattern.
        let _ = listen.call3(
            &JsValue::UNDEFINED,
            &JsValue::from_str("deep-link://new-url"),
            callback_ptr,
            &JsValue::UNDEFINED,
        );
    }
    callback.forget();
}

fn route_shortcut(url: &str) {
    if let Some(source) = parse_shortcut_url(url) {
        park_shortcut_tab(source.tab_id());
    }
}

fn spawn_local_or_ignore(future: impl std::future::Future<Output = ()> + 'static) {
    #[cfg(target_arch = "wasm32")]
    leptos::task::spawn_local(async move {
        future.await;
    });
    #[cfg(not(target_arch = "wasm32"))]
    drop(future);
}
