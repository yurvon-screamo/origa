//! Umami Cloud analytics: runtime tracker injection.
//!
//! The tracker `<script>` is injected from WASM instead of being hardcoded in
//! `index.html` so that CI builds can compile it out: `build.rs` emits an
//! empty `UMAMI_WEBSITE_ID` when `UMAMI_DISABLED=1` is set (e2e headless runs,
//! android-smoke emulator), and an empty value makes [`inject_umami`] a no-op.
//! A static tag cannot be muted per build and would leak CI's
//! localhost/tauri.localhost traffic into production analytics. See ADR-054.

/// Umami Cloud tracker script. The host is pinned exactly (no wildcard) — it
/// is also allow-listed verbatim in the Tauri CSP (`tauri/build_config.rs::
/// build_csp`), where the same origin executes with Tauri IPC in reach.
pub(crate) const UMAMI_SCRIPT_SRC: &str = "https://cloud.umami.is/script.js";

/// Inject the Umami tracker `<script>` into `<head>` before the app
/// mounts, so auto-tracking catches the initial pageview and (the tracker
/// patches the History API) every SPA navigation afterwards.
///
/// No-op when analytics was compiled out ([`analytics_enabled`]) or the
/// DOM is unavailable. Injection failure is non-fatal by design — losing
/// analytics must never break the app — so errors are logged, not propagated.
///
/// Note: `defer` is intentionally NOT set — it only affects parser-inserted
/// tags, and this script is inserted at runtime. Execution order is
/// irrelevant for the tracker (it reports the current URL when it runs).
pub(crate) fn inject_umami() {
    if !analytics_enabled() {
        return;
    }
    let website_id = env!("UMAMI_WEBSITE_ID");

    let Some(document) = web_sys::window().and_then(|w| w.document()) else {
        return;
    };
    let Some(head) = document.head() else {
        return;
    };

    let script = document.create_element("script");
    let Ok(script) = script else {
        tracing::warn!("Umami tracker: <script> creation failed");
        return;
    };

    let attrs = [
        ("src", UMAMI_SCRIPT_SRC),
        ("data-website-id", website_id),
        ("data-domains", &umami_data_domains()),
        ("data-do-not-track", "true"),
    ];
    for (name, value) in attrs {
        if script.set_attribute(name, value).is_err() {
            tracing::warn!("Umami tracker: setting attribute {name} failed");
            return;
        }
    }

    if head.append_child(&script).is_err() {
        tracing::warn!("Umami tracker: appending to <head> failed");
    }
}

/// Single mute predicate: analytics is compiled out when `build.rs` emitted
/// an empty `UMAMI_WEBSITE_ID` (`UMAMI_DISABLED=1` — e2e headless runs,
/// android-smoke emulator). Every analytics entry point (tracker injection,
/// programmatic events) gates on this one function.
pub(crate) fn analytics_enabled() -> bool {
    !env!("UMAMI_WEBSITE_ID").is_empty()
}

/// Fire-and-forget programmatic event through the injected tracker's
/// `umami.track(name, data)` contract (ADR-054 §5, programmatic slice).
///
/// Delivery is best-effort by design — losing analytics must never break
/// the app: a failed call is logged and dropped. When the tracker has not
/// executed yet (it loads asynchronously after injection, and events can
/// fire within the first hundreds of milliseconds of a page — cold start,
/// the resume jump to Scoring), the event goes into a bounded pending
/// queue flushed as soon as `window.umami` appears ([`enqueue_pending`]).
///
/// `props` become the tracker's event data object (visible in Umami's
/// event properties), so values must be strings per the tracker contract.
pub(crate) fn track_event(name: &str, props: &[(&str, &str)]) {
    use wasm_bindgen::JsCast;

    if !analytics_enabled() {
        return;
    }
    let Some(window) = web_sys::window() else {
        return;
    };

    let data = js_sys::Object::new();
    for (key, value) in props {
        let _set = js_sys::Reflect::set(
            &data,
            &wasm_bindgen::JsValue::from_str(key),
            &wasm_bindgen::JsValue::from_str(value),
        );
    }

    let umami = match js_sys::Reflect::get(&window, &wasm_bindgen::JsValue::from_str("umami")) {
        Ok(umami) if !umami.is_undefined() && !umami.is_null() => umami,
        _ => {
            enqueue_pending(&window, name, &data);
            return;
        },
    };

    let track_fn = match js_sys::Reflect::get(&umami, &wasm_bindgen::JsValue::from_str("track")) {
        Ok(track) => match track.dyn_into::<js_sys::Function>() {
            Ok(track_fn) => track_fn,
            Err(_) => {
                tracing::warn!("Umami: window.umami.track is not a function, dropped {name}");
                return;
            },
        },
        Err(_) => {
            tracing::warn!("Umami: window.umami.track missing, dropped event {name}");
            return;
        },
    };

    // `this` must be the `umami` object: the tracker exposes track as an
    // object method, and a detached call can lose its receiver.
    if let Err(error) = track_fn.call2(&umami, &wasm_bindgen::JsValue::from_str(name), &data) {
        tracing::warn!("Umami: track call failed for {name}: {error:?}");
    }
}

/// Poll interval and give-up bound of the pending-event flusher: 50 × 200 ms
/// = 10 s. Past the bound the tracker is considered absent (ad blocker,
/// muted build) and pending events are discarded instead of queueing forever.
const PENDING_FLUSH_INTERVAL_MS: u32 = 200;
const PENDING_FLUSH_MAX_TRIES: u32 = 50;

/// Queues an event for `window.umami` and installs the one-shot flusher
/// (idempotent). The flusher lives entirely in this JS snippet: on each tick
/// it waits for a usable tracker, then drains the queue in FIFO order —
/// preserving event order — and clears itself. When the give-up bound is
/// reached it discards the queue instead of holding it forever.
fn enqueue_pending(window: &web_sys::Window, name: &str, data: &js_sys::Object) {
    let enqueue = js_sys::Function::new_with_args(
        "name, data",
        // The trailing statement runs on every call; the guarded block only
        // on the first (window.__umami_enqueue is the installed singleton).
        r#"
            if (!window.__umami_enqueue) {
                window.__umami_pending = [];
                window.__umami_enqueue = function (name, data) {
                    window.__umami_pending.push({ name: name, data: data });
                    if (window.__umami_flush_timer) return;
                    window.__umami_flush_tries = 0;
                    window.__umami_flush_timer = setInterval(function () {
                        window.__umami_flush_tries++;
                        if (!(window.umami && typeof window.umami.track === 'function')) {
                            if (window.__umami_flush_tries >= {max_tries}) {
                                clearInterval(window.__umami_flush_timer);
                                window.__umami_flush_timer = null;
                                window.__umami_pending = [];
                            }
                            return;
                        }
                        clearInterval(window.__umami_flush_timer);
                        window.__umami_flush_timer = null;
                        var pending = window.__umami_pending.splice(0);
                        for (var i = 0; i < pending.length; i++) {
                            try {
                                window.umami.track(pending[i].name, pending[i].data);
                            } catch (e) {}
                        }
                    }, {interval_ms});
                };
            }
            window.__umami_enqueue(name, data);
        "#
        .replace("{max_tries}", &PENDING_FLUSH_MAX_TRIES.to_string())
        .replace("{interval_ms}", &PENDING_FLUSH_INTERVAL_MS.to_string())
        .as_str(),
    );
    if let Err(error) = enqueue.call2(window, &wasm_bindgen::JsValue::from_str(name), data) {
        tracing::warn!("Umami: enqueue of {name} failed: {error:?}");
    }
}

/// `data-domains` hostname allow-list: the production web app plus the
/// origins Tauri serves desktop builds from — Windows/Android
/// `tauri.localhost`, Linux `http://localhost` and macOS `tauri://localhost`
/// (both normalize to hostname `localhost`). Any other host (dev mirrors,
/// staging) is silently not tracked.
///
/// The web app host is derived from `TRAILBASE_URL` rather than hardcoded so
/// the domain has a single source of truth (`build_defaults.rs`) — the two
/// are deployed on the same host.
fn umami_data_domains() -> String {
    let trailbase = env!("TRAILBASE_URL");
    let without_scheme = trailbase
        .trim_start_matches("https://")
        .trim_start_matches("http://");
    // Strip an optional port; `TRAILBASE_URL` is scheme://host[:port] with no
    // path in every supported configuration.
    let app_host = match without_scheme.split_once(':') {
        Some((host, _)) => host,
        None => without_scheme,
    };
    format!("{app_host},tauri.localhost,localhost")
}
