//! Share-intake frontend bridge (IN-2/IN-3): receives externally shared
//! content (Android share sheet, desktop file association) from the host
//! and parks it for the add-words inbox.
//!
//! Wire payloads arrive via the `origa://share-intake` event (warm) or
//! the `get_pending_share` command (cold start). File bytes live in the
//! host cache and are fetched with `read_share_file` (one invoke cycle)
//! when the inbox consumes the payload. The pending slot is take-once:
//! the Words page consumes it on mount; a newer share overwrites an
//! unconsumed older one.

use base64::{Engine, engine::general_purpose::STANDARD};
use leptos::prelude::*;
use wasm_bindgen::{JsCast, JsValue};

use crate::core::tauri::{event_listen_fn, invoke_with_args, is_tauri};

/// Wire payload mirrored from the host (`tauri/src/share_intake.rs`).
#[derive(Debug, Clone, Default, PartialEq)]
pub enum ShareWire {
    #[default]
    None,
    Text {
        text: String,
    },
    File {
        file_name: String,
        mime: String,
        cache_path: String,
    },
    Error {
        message: String,
    },
}

impl ShareWire {
    /// Decodes a host event payload (camelCase, adjacent tag).
    fn from_js(value: &JsValue) -> Option<Self> {
        let kind = js_sys::Reflect::get(value, &JsValue::from_str("kind"))
            .ok()?
            .as_string()?;
        match kind.as_str() {
            "text" => {
                let text = js_sys::Reflect::get(value, &JsValue::from_str("text"))
                    .ok()?
                    .as_string()?;
                Some(ShareWire::Text { text })
            },
            "file" => {
                let file_name = js_sys::Reflect::get(value, &JsValue::from_str("fileName"))
                    .ok()?
                    .as_string()?;
                let mime = js_sys::Reflect::get(value, &JsValue::from_str("mime"))
                    .ok()?
                    .as_string()
                    .unwrap_or_default();
                let cache_path = js_sys::Reflect::get(value, &JsValue::from_str("cachePath"))
                    .ok()?
                    .as_string()?;
                Some(ShareWire::File {
                    file_name,
                    mime,
                    cache_path,
                })
            },
            "error" => {
                let message = js_sys::Reflect::get(value, &JsValue::from_str("message"))
                    .ok()?
                    .as_string()?;
                Some(ShareWire::Error { message })
            },
            _ => None,
        }
    }
}

/// Global take-once parking slot (the last share wins).
static PENDING: std::sync::OnceLock<RwSignal<Option<ShareWire>>> = std::sync::OnceLock::new();

fn pending_slot() -> &'static RwSignal<Option<ShareWire>> {
    PENDING.get_or_init(|| RwSignal::new(None))
}

/// Parks a payload for the Words page to consume on mount.
pub fn park_share(payload: ShareWire) {
    pending_slot().set(Some(payload));
}

/// Read-only signal access for navigation triggers.
pub fn pending_signal() -> RwSignal<Option<ShareWire>> {
    *pending_slot()
}

/// Takes the parked share (read + clear atomically).
pub fn take_share() -> Option<ShareWire> {
    let slot = pending_slot();
    let value = slot.get_untracked();
    if value.is_some() {
        slot.set(None);
    }
    value
}

/// Starts the host-event listener and polls the cold-start pending slot.
/// Web build: no Tauri bridge → silent no-op (the updater-listener
/// pattern). Returns false when no bridge exists (for tests).
pub fn start_share_intake() -> bool {
    if !is_tauri() {
        return false;
    }
    // Cold start: the event fired before this listener existed.
    poll_pending_share();
    listen_share_events();
    // Android warm shares arrive with window focus (no event channel).
    poll_on_focus();
    true
}

fn poll_pending_share() {
    spawn_local_or_ignore(async move {
        match invoke_with_args("get_pending_share", &JsValue::UNDEFINED).await {
            Ok(value) => {
                if let Some(payload) = ShareWire::from_js(&value) {
                    park_share(payload);
                }
            },
            Err(e) => tracing::debug!("share-intake: pending poll failed: {e}"),
        }
    });
}

/// Re-polls the host pending slot: Android warm shares park in the Kotlin
/// ShareBuffer (no event channel), so window focus (= app brought to
/// foreground by a share) must re-check.
fn poll_on_focus() {
    let Some(window) = web_sys::window() else {
        return;
    };
    let callback = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
        poll_pending_share();
    }) as Box<dyn Fn()>);
    let callback_ptr: &js_sys::Function = callback
        .as_ref()
        .dyn_ref::<js_sys::Function>()
        .expect("closure to Function");
    let _ = window.add_event_listener_with_callback("focus", callback_ptr);
    callback.forget();
}

fn listen_share_events() {
    let Some(listen) = event_listen_fn() else {
        return;
    };
    let callback = wasm_bindgen::closure::Closure::wrap(Box::new(move |event: JsValue| {
        let payload = js_sys::Reflect::get(&event, &JsValue::from_str("payload"))
            .ok()
            .and_then(|value| ShareWire::from_js(&value));
        if let Some(payload) = payload {
            park_share(payload);
        }
    }) as Box<dyn FnMut(JsValue)>);
    let Some(callback_ptr) = callback.as_ref().dyn_ref::<js_sys::Function>() else {
        tracing::warn!("share-intake: closure to Function conversion failed");
        return;
    };
    let _ = listen.call3(
        &JsValue::UNDEFINED,
        &JsValue::from_str("origa://share-intake"),
        callback_ptr,
        &JsValue::UNDEFINED,
    );
    // The listener lives for the app lifetime; the closure must not be
    // dropped (leaks once by design — same as the oauth listeners).
    callback.forget();
}

fn spawn_local_or_ignore(future: impl std::future::Future<Output = ()> + 'static) {
    #[cfg(target_arch = "wasm32")]
    leptos::task::spawn_local(async move {
        future.await;
    });
    #[cfg(not(target_arch = "wasm32"))]
    drop(future);
}

/// Reads the parked shared file's bytes through the host command (one
/// invoke cycle) and deletes the cache file afterwards.
pub async fn read_shared_bytes(cache_path: &str) -> Result<Vec<u8>, String> {
    let args = js_sys::Object::new();
    js_sys::Reflect::set(
        &args,
        &JsValue::from_str("path"),
        &JsValue::from_str(cache_path),
    )
    .map_err(|_| "share-intake args: path".to_string())?;
    let raw = invoke_with_args("read_share_file", &args.into()).await?;
    let base64 = raw
        .as_string()
        .ok_or("share-intake: read response is not a string")?;
    let bytes = STANDARD
        .decode(base64.as_bytes())
        .map_err(|e| format!("share-intake: base64 decode failed: {e:?}"))?;

    // Best-effort cleanup of the consumed cache file.
    let delete_args = js_sys::Object::new();
    let _ = js_sys::Reflect::set(
        &delete_args,
        &JsValue::from_str("path"),
        &JsValue::from_str(cache_path),
    );
    let _ = invoke_with_args("delete_share_file", &delete_args.into()).await;

    Ok(bytes)
}

#[cfg(all(target_arch = "wasm32", test))]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn wire_from_js_decodes_all_variants() {
        fn js_object(pairs: &[(&str, &str)]) -> JsValue {
            let obj = js_sys::Object::new();
            for (key, value) in pairs {
                let _ =
                    js_sys::Reflect::set(&obj, &JsValue::from_str(key), &JsValue::from_str(value));
            }
            obj.into()
        }

        let text = js_object(&[("kind", "text"), ("text", "本を読む")]);
        assert!(
            matches!(ShareWire::from_js(&text), Some(ShareWire::Text { ref text }) if text == "本を読む"),
            "text variant: {:?}",
            ShareWire::from_js(&text)
        );

        let file = js_object(&[
            ("kind", "file"),
            ("fileName", "a.png"),
            ("mime", "image/png"),
            ("cachePath", "/tmp/x"),
        ]);
        assert_eq!(
            ShareWire::from_js(&file),
            Some(ShareWire::File {
                file_name: "a.png".to_string(),
                mime: "image/png".to_string(),
                cache_path: "/tmp/x".to_string(),
            })
        );

        let error = js_object(&[("kind", "error"), ("message", "boom")]);
        assert_eq!(
            ShareWire::from_js(&error),
            Some(ShareWire::Error {
                message: "boom".to_string()
            })
        );

        assert_eq!(ShareWire::from_js(&JsValue::UNDEFINED), None);
        let unknown = js_object(&[("kind", "surprise")]);
        assert_eq!(ShareWire::from_js(&unknown), None);
    }

    #[wasm_bindgen_test]
    fn pending_slot_take_once_last_wins() {
        // wasm tests run interleaved: reset the slot before asserting.
        let _ = take_share();
        park_share(ShareWire::Text {
            text: "first".to_string(),
        });
        park_share(ShareWire::Text {
            text: "second".to_string(),
        });
        assert!(
            matches!(take_share(), Some(ShareWire::Text { text }) if text == "second"),
            "last write wins"
        );
        assert!(take_share().is_none(), "second take is empty");
    }
}
