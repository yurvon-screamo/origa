//! Sentry integration for the WASM frontend.
//!
//! Sentry's JavaScript SDK is used (the Rust `sentry` crate does not compile
//! to WASM). The loader script is injected dynamically at runtime via the DOM
//! (`<script src="https://js.sentry-cdn.com/{public_key}.min.js">`) rather than
//! hardcoded in `index.html`, so the DSN can be parameterised at build time
//! through `SENTRY_DSN_UI` without violating the Cargo "build scripts must not
//! mutate committed source files" contract. See ADR-036 §4.
//!
//! The `layer = "ui"` scope tag distinguishes WASM/browser events from
//! tauri-native events in the shared Sentry project.
//!
//! ## Panic bridge
//!
//! `init()` does not install the panic hook itself — `lib.rs::init_tracing`
//! wraps the existing `console_error_panic_hook` so that Rust panics are first
//! forwarded to `capture_exception` and then logged to the browser console.
//! On the WASM side there is no `flush(None)` concern: the JS SDK flushes
//! asynchronously, so the panic hook never blocks.

use js_sys::Reflect;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Document, Element};

/// Build-time-injected Sentry configuration. Empty values disable Sentry
/// (the dev/Dependabot path). See `build.rs`.
const DSN: &str = env!("SENTRY_DSN_UI");

/// Compile-time Sentry DSN for the UI layer (ADR-055). Empty = Sentry is
/// compiled out (dev/e2e builds) — the only legitimate trigger of the
/// feedback form's "available in release builds" info state.
pub fn dsn_ui() -> &'static str {
    DSN
}
const RELEASE: &str = env!("SENTRY_RELEASE_UI");
const ENVIRONMENT: &str = env!("SENTRY_ENVIRONMENT_UI");

/// Initialise Sentry by injecting the loader script and configuring the SDK.
///
/// No-op when `SENTRY_DSN_UI` is empty (dev/Dependabot builds). Defensive
/// against missing `window`/`document` (SSR contexts, headless test runners).
///
/// The actual `Sentry.init` call is deferred until the loader script has
/// loaded via its `window.sentryOnLoad` callback hook, which we install
/// *before* injecting the script to avoid losing the callback if the script
/// is served from HTTP cache and loads synchronously (ADR-036 §4).
pub fn init() {
    init_with(DSN, RELEASE, ENVIRONMENT);
}

/// Parameterised entry point used by `init()` and the wasm-bindgen tests.
/// Split out so tests can drive the loader-injection logic with arbitrary
/// DSN/release/environment values without depending on compile-time env.
pub(crate) fn init_with(dsn: &str, release: &str, environment: &str) {
    if dsn.is_empty() {
        tracing::debug!("[sentry] disabled (no SENTRY_DSN_UI)");
        return;
    }

    let Some(public_key) = extract_public_key(dsn) else {
        tracing::warn!("[sentry] could not parse public key from DSN, disabling");
        return;
    };

    let Some(window) = web_sys::window() else {
        tracing::warn!("[sentry] no global window, skipping init");
        return;
    };
    let Some(document) = window.document() else {
        tracing::warn!("[sentry] no document, skipping init");
        return;
    };
    let Some(head) = document.head() else {
        tracing::warn!("[sentry] no <head>, skipping init");
        return;
    };

    // 1. Install `window.sentryOnLoad` BEFORE injecting the script so the
    //    loader invokes it even when it loads synchronously from cache.
    //
    //    Background flush: iOS/Android suspend the WebView on
    //    backgrounding, freezing the SDK's async transport — queued
    //    events die unflushed (the exact blind spot that hid the
    //    2026-09 sync-error reports from RC builds). The listeners are
    //    registered immediately (NOT inside `sentryOnLoad`): before the
    //    loader finishes, `window.Sentry` is undefined and the flush is a
    //    no-op; once loaded, `visibilitychange → hidden` and `pagehide`
    //    both push the queue out while the page is still alive.
    let init_js = build_init_js(dsn, release, environment);
    if let Err(e) = js_sys::eval(&init_js) {
        tracing::warn!("[sentry] failed to install sentryOnLoad: {:?}", e);
        return;
    }

    // 2. Inject the loader script. `defer` is intentionally NOT set: it is a
    //    no-op on dynamically-inserted <script> elements per HTML spec, and
    //    `sentryOnLoad` above already gates the actual init.
    let loader_url = format!("https://js.sentry-cdn.com/{public_key}.min.js");
    if let Err(e) = inject_script(&document, &head, &loader_url) {
        tracing::warn!("[sentry] failed to inject loader script: {:?}", e);
        return;
    }

    tracing::info!(
        "[sentry] enabled (release={}, environment={})",
        release,
        environment
    );
}

/// The `beforeSend` predicate of the bootstrap init: stack-less WASM traps
/// group poorly — every "RuntimeError: unreachable" lands in one card so
/// one failure class is one issue (page context stays on the event's tags).
/// SDK event shape: the error class is in `type`, the message in `value`
/// (verified against live RUST-4/R events). A standalone const so the wasm
/// test suite evaluates the actual function object against SDK-shaped mock
/// events — a substring match on the generated JS proves nothing about
/// behavior.
pub(crate) const BEFORE_SEND_FN: &str = r#"function(event) {
    try {
        var v = event && event.exception
            && event.exception.values
            && event.exception.values[0] || null;
        if (v && v.type === 'RuntimeError' && v.value === 'unreachable') {
            event.fingerprint = ['wasm-unreachable'];
        }
    } catch (e) { /* never drop an event over grouping */ }
    return event;
}"#;

/// Build the `sentryOnLoad` bootstrap JS executed before the loader script
/// is injected (ADR-036 §4: installed BEFORE the script tag so a
/// cache-served synchronous load cannot miss the callback). Error delivery
/// does NOT go through a console capture integration — the full story
/// lives in the ADR-036 addendum. The `beforeSend` fingerprints the
/// stack-less `RuntimeError: unreachable` family (RUST-4/R) into a single
/// group: raw frames are `wasm-function[N]` addresses without symbols, and
/// page-level signal stays on the event's `transaction`/`url` tags.
pub(crate) fn build_init_js(dsn: &str, release: &str, environment: &str) -> String {
    format!(
        r#"(function() {{
            if (window.__sentryFlushInstalled) {{
                return;
            }}
            window.__sentryFlushInstalled = true;
            var flushPending = function() {{
                // Own attempt counter: hermetic for tests (the loader from a
                // prior init can replace `window.Sentry` with the real SDK
                // at any async point) and a cheap breadcrumb for debugging.
                window.__sentryFlushAttempts = (window.__sentryFlushAttempts || 0) + 1;
                try {{
                    if (window.Sentry && typeof window.Sentry.flush === 'function') {{
                        window.Sentry.flush(2000);
                    }}
                }} catch (e) {{ /* never throw from a lifecycle handler */ }}
            }};
            document.addEventListener('visibilitychange', function() {{
                if (document.visibilityState === 'hidden') {{
                    flushPending();
                }}
            }});
            window.addEventListener('pagehide', flushPending);
            window.sentryOnLoad = function() {{
                // Delivery story (direct captureException from the tracing
                // layer, no console capture) — ADR-036 addendum.
                Sentry.init({{
                    dsn: "{dsn}",
                    release: "{release}",
                    environment: "{environment}",
                    sendDefaultPii: false,
                    tracesSampleRate: 1.0,
                    enableLogs: true,
                    beforeSend: {before_send}
                }});
                Sentry.setTag("layer", "ui");
            }};
        }})();"#,
        before_send = BEFORE_SEND_FN
    )
}

/// Forward a message to Sentry as a captured exception.
///
/// Two callers: the panic hook (every WASM panic) and the
/// `tracing_console` layer (every routed `error!` event).
///
/// **Must never panic itself**: this is called from the panic hook on every
/// WASM panic, including the path where Sentry is disabled (no loader
/// injected, `window.Sentry` is `undefined`). Defensive on every JS call —
/// any failure is silently dropped so the downstream console hook still runs.
///
/// Two things are critical here:
///
/// 1. **Wrap in `new Error(msg)`** — Sentry groups events by stack trace.
///    Passing a bare string collapses every panic into a single issue group
///    in the dashboard, making them indistinguishable. `js_sys::Error::new`
///    captures a synthetic JS stack at this call site, which Sentry uses for
///    deduplication and grouping.
///
/// 2. **Call `Sentry.flush(2000)`** — `captureException` only queues the
///    event; the actual HTTP POST to the ingest endpoint happens on the next
///    event-loop tick. After `panic = "abort"` fires the WASM `unreachable`
///    trap, the JS runtime stays alive but the running async task is dead.
///    Without an explicit flush the queued event may sit in the SDK buffer
///    indefinitely. `flush` returns a Promise — we fire it without awaiting
///    (the panic hook is synchronous), but calling it starts the send
///    immediately.
pub fn capture_exception(msg: &str) {
    let Ok(sentry_obj) = sentry_global() else {
        return;
    };
    if sentry_obj.is_undefined() || sentry_obj.is_null() {
        return;
    }

    // Wrap in a real Error so Sentry has a stacktrace for grouping.
    let error = js_sys::Error::new(msg);

    let Ok(capture) = Reflect::get(&sentry_obj, &"captureException".into()) else {
        return;
    };
    let Some(capture_fn) = capture.dyn_ref::<js_sys::Function>() else {
        return;
    };
    let _ = capture_fn.call1(&sentry_obj, &error);

    // Force the SDK to send the queued event immediately. Without this the
    // event may never leave the buffer if the WASM trap cascades.
    if let Ok(flush) = Reflect::get(&sentry_obj, &"flush".into()) {
        if let Some(flush_fn) = flush.dyn_ref::<js_sys::Function>() {
            let _ = flush_fn.call1(&sentry_obj, &JsValue::from_f64(2000.0));
        }
    }
}

/// Read `window.Sentry`. Returns `Err` only if `window` itself is missing;
/// the returned value may still be `undefined` when the loader has not yet
/// loaded (or was never injected).
fn sentry_global() -> Result<JsValue, JsValue> {
    let window = web_sys::window().ok_or_else(|| JsValue::from("no window"))?;
    Reflect::get(&window.into(), &"Sentry".into())
}

/// Extract the public key (first URL segment before `@`) from a Sentry DSN.
///
/// DSN format: `https://<public_key>@<host>/<project_id>`. Returns `None` for
/// malformed inputs so `init_with` can fail gracefully.
fn extract_public_key(dsn: &str) -> Option<&str> {
    let before_at = dsn.split_once('@')?.0;
    let key = before_at
        .rsplit_once("://")
        .map(|(_, k)| k)
        .unwrap_or(before_at);
    if key.is_empty() {
        return None;
    }
    Some(key)
}

/// Create and append a `<script src=url crossorigin data-lazy=no>` element to
/// `<head>`.
///
/// `data-lazy="no"` forces the loader to fetch the full SDK immediately (on the
/// next event-loop tick) instead of waiting for the first error. This is
/// required for performance monitoring: BrowserTracing must be active *before*
/// the app's `fetch` calls to capture them as transactions. Without it, the
/// loader only downloads the SDK when an error occurs — by which point the
/// dictionary-loading `fetch` requests have already happened uninstrumented.
fn inject_script(
    document: &Document,
    head: &web_sys::HtmlHeadElement,
    url: &str,
) -> Result<(), JsValue> {
    let script: Element = document.create_element("script")?;
    script.set_attribute("src", url)?;
    script.set_attribute("crossorigin", "anonymous")?;
    script.set_attribute("data-lazy", "no")?;
    head.append_child(&script)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_public_key_parses_standard_dsn() {
        let dsn = "https://abc123def456@o789.ingest.sentry.io/42";
        assert_eq!(extract_public_key(dsn), Some("abc123def456"));
    }

    #[test]
    fn extract_public_key_rejects_missing_at() {
        assert_eq!(extract_public_key("https://sentry.io/42"), None);
    }

    #[test]
    fn extract_public_key_rejects_empty_key() {
        assert_eq!(extract_public_key("https://@o1.ingest.sentry.io/1"), None);
    }

    #[test]
    fn extract_public_key_handles_missing_scheme() {
        // Defensive: a DSN missing the scheme still has the key segment,
        // though Sentry itself would reject it. We parse leniently here and
        // let Sentry's own init fail if the value is unusable.
        assert_eq!(extract_public_key("abc@host/1"), Some("abc"));
    }

    #[test]
    fn extract_public_key_rejects_empty_dsn() {
        assert_eq!(extract_public_key(""), None);
    }

    /// The bootstrap JS carries the `beforeSend` fingerprint for the
    /// stack-less `RuntimeError: unreachable` family (uwuwu#1454): without
    /// it Sentry splits one failure class across sibling issues (RUST-4/R
    /// in the 30-day audit window).
    #[test]
    fn init_js_contains_unreachable_fingerprint() {
        let js = build_init_js(
            "https://abc123def456@o789.ingest.sentry.io/42",
            "1.0.0",
            "test",
        );
        assert!(
            js.contains("beforeSend: function(event)"),
            "beforeSend must be wired into init"
        );
        assert!(
            js.contains("v.type === 'RuntimeError' && v.value === 'unreachable'"),
            "fingerprint trigger must match the SDK event shape (type/value)"
        );
        assert!(
            js.contains("['wasm-unreachable']"),
            "fingerprint family must be pinned"
        );
    }

    /// The dead console-capture path must be GONE from the bootstrap: SDK
    /// v10 lazy-loads `captureConsoleIntegration`, so the `typeof` guard
    /// silently produced an empty integrations list for months (uwuwu#1454,
    /// delta-round 2). Error delivery goes through the direct
    /// `captureException` bridge in the tracing layer — the bootstrap must
    /// not reference the integration at all.
    #[test]
    fn init_js_has_no_console_capture_path() {
        let js = build_init_js(
            "https://abc123def456@o789.ingest.sentry.io/42",
            "1.0.0",
            "test",
        );
        assert!(
            !js.contains("captureConsoleIntegration"),
            "captureConsoleIntegration must not be configured (dead in SDK v10)"
        );
    }

    /// DSN/release/environment interpolate into the bootstrap config.
    #[test]
    fn init_js_interpolates_config() {
        let js = build_init_js("https://key@host/42", "9.9.9", "staging");
        assert!(js.contains("dsn: \"https://key@host/42\""));
        assert!(js.contains("release: \"9.9.9\""));
        assert!(js.contains("environment: \"staging\""));
    }
}

/// WASM-only tests for `init_with`'s DOM-injection logic. Run via
/// `wasm-pack test --chrome origa_ui -- sentry` (or
/// `cargo test -p origa_ui --target wasm32-unknown-unknown -- sentry`).
///
/// These are the regression guards for the race fixed in ADR-036 §4
/// (`sentryOnLoad` installed BEFORE `<script>` appended) and for the
/// empty-DSN no-op contract.
#[cfg(all(target_arch = "wasm32", test))]
mod wasm_tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    /// Empty DSN must not inject any `<script>` into `<head>`. Regression
    /// guard for the dev/Dependabot disabled-Sentry path.
    #[wasm_bindgen_test]
    fn init_with_empty_dsn_is_noop() {
        let before = count_head_scripts();
        init_with("", "1.0.0", "test");
        let after = count_head_scripts();
        assert_eq!(
            before, after,
            "no <script> should be injected for empty DSN"
        );
    }

    /// Behavior test for the `beforeSend` predicate (uwuwu#1454 review): the
    /// actual function object runs against SDK-shaped mock events. The live
    /// RUST-4/R events carry `type='RuntimeError'`, `value='unreachable'` —
    /// exactly the shape fingerprinted here; everything else must pass
    /// through untouched.
    #[wasm_bindgen_test]
    fn before_send_fingerprints_real_unreachable_shape() {
        let f = js_sys::eval(&format!("({})", BEFORE_SEND_FN))
            .expect("beforeSend evaluates")
            .dyn_into::<js_sys::Function>()
            .expect("beforeSend is a function");

        let fingerprint_of = |json: &str| -> Option<String> {
            let ev = js_sys::JSON::parse(json).expect("mock event parses");
            let out = f.call1(&JsValue::UNDEFINED, &ev).expect("beforeSend runs");
            let fp = js_sys::Reflect::get(&out, &"fingerprint".into()).ok()?;
            let arr = fp.dyn_into::<js_sys::Array>().ok()?;
            arr.get(0).as_string()
        };

        // Live RUST-4/R shape → fingerprinted.
        assert_eq!(
            fingerprint_of(
                r#"{"exception":{"values":[{"type":"RuntimeError","value":"unreachable"}]}}"#
            )
            .as_deref(),
            Some("wasm-unreachable"),
            "the real unreachable shape must be fingerprinted"
        );
        // Same class, different message → untouched.
        assert_eq!(
            fingerprint_of(
                r#"{"exception":{"values":[{"type":"RuntimeError","value":"out of memory"}]}}"#
            ),
            None
        );
        // Same message, different class → untouched.
        assert_eq!(
            fingerprint_of(
                r#"{"exception":{"values":[{"type":"TypeError","value":"unreachable"}]}}"#
            ),
            None
        );
        // No exception block → untouched.
        assert_eq!(fingerprint_of(r#"{"message":"plain"}"#), None);
    }

    /// A valid DSN must inject exactly one `<script>` with the expected
    /// loader URL into `<head>`. Regression guard for the loader-injection
    /// path (extract_public_key + inject_script + sentryOnLoad ordering).
    #[wasm_bindgen_test]
    fn init_with_dsn_injects_single_loader_script() {
        // Snapshot before so the test is order-independent across other
        // tests that may have already injected.
        let before = count_head_scripts();
        init_with(
            "https://abc123def456@o789.ingest.sentry.io/42",
            "1.0.0",
            "test",
        );
        let after = count_head_scripts();

        assert_eq!(
            after - before,
            1,
            "exactly one <script> should be injected for a set DSN"
        );

        // The injected script's src must point at the Sentry loader CDN with
        // the public key extracted from the DSN.
        let last_src = last_head_script_src().unwrap_or_default();
        assert!(
            last_src.contains("js.sentry-cdn.com/abc123def456.min.js"),
            "injected script src must point at the Sentry loader CDN with the DSN public key, got: {last_src}"
        );

        // data-lazy="no" must be set so the loader fetches the full SDK
        // immediately (required for BrowserTracing to capture fetch calls
        // during dictionary loading).
        let last_data_lazy = last_head_script_data_lazy().unwrap_or_default();
        assert_eq!(
            last_data_lazy, "no",
            "injected script must have data-lazy=no for eager SDK loading"
        );
    }

    fn count_head_scripts() -> usize {
        let window = web_sys::window().expect("window");
        let document = window.document().expect("document");
        let head = document.head().expect("head");
        head.query_selector_all("script")
            .map(|nodes| nodes.length() as usize)
            .unwrap_or(0)
    }

    fn last_head_script_src() -> Option<String> {
        let window = web_sys::window()?;
        let document = window.document()?;
        let head = document.head()?;
        let nodes = head.query_selector_all("script").ok()?;
        let last = nodes.item(nodes.length() - 1)?;
        let el = last.dyn_ref::<web_sys::Element>()?;
        el.get_attribute("src")
    }

    fn last_head_script_data_lazy() -> Option<String> {
        let window = web_sys::window()?;
        let document = window.document()?;
        let head = document.head()?;
        let nodes = head.query_selector_all("script").ok()?;
        let last = nodes.item(nodes.length() - 1)?;
        let el = last.dyn_ref::<web_sys::Element>()?;
        el.get_attribute("data-lazy")
    }

    /// Reads the listener's own flush-attempt counter (set by
    /// `flushPending` in the injected init script).
    fn flush_attempts() -> u32 {
        js_sys::eval("window.__sentryFlushAttempts || 0")
            .ok()
            .and_then(|v| v.as_f64())
            .map(|v| v as u32)
            .unwrap_or(0)
    }

    fn dispatch_event(target: &str, event_type: &str) {
        let js = format!(
            "(function() {{ var e = new Event('{event_type}'); \
               ({target}).dispatchEvent(e); }})();"
        );
        js_sys::eval(&js).expect("event dispatch eval");
    }

    /// `pagehide` (iOS termination path) must reach `Sentry.flush` on the
    /// live `window.Sentry`. Asserts both the listener's own attempt
    /// counter (hermetic — unaffected by any SDK replacement) and the
    /// stubbed flush call itself (the loader URL uses a fake key, so no
    /// real SDK ever replaces the stub in the test browser).
    #[wasm_bindgen_test]
    fn background_flush_fires_on_pagehide() {
        init_with(
            "https://abc123def456@o789.ingest.sentry.io/42",
            "1.0.0",
            "test",
        );
        js_sys::eval(
            "window.__stubFlushCount = 0; \
             window.Sentry = { flush: function() { window.__stubFlushCount++; } };",
        )
        .expect("flush stub eval");
        let before = flush_attempts();

        dispatch_event("window", "pagehide");

        assert_eq!(
            flush_attempts(),
            before + 1,
            "pagehide must run the background flush"
        );
        let stub_calls = js_sys::eval("window.__stubFlushCount || 0")
            .ok()
            .and_then(|v| v.as_f64())
            .map(|v| v as u32)
            .unwrap_or(0);
        assert_eq!(stub_calls, 1, "the flush must call window.Sentry.flush");
    }

    /// `visibilitychange` while the document is still visible must NOT
    /// flush — the handler gates on `document.visibilityState === 'hidden'`.
    #[wasm_bindgen_test]
    fn background_flush_skips_while_visible() {
        init_with(
            "https://abc123def456@o789.ingest.sentry.io/42",
            "1.0.0",
            "test",
        );
        let before = flush_attempts();

        dispatch_event("document", "visibilitychange");

        assert_eq!(
            flush_attempts(),
            before,
            "visibilitychange while visible must not flush"
        );
    }

    /// Poll `window.Sentry` until the loader-served SDK is up.
    async fn wait_sentry(timeout_polls: u32) -> bool {
        for _ in 0..timeout_polls {
            if let Ok(g) = sentry_global() {
                if !g.is_undefined() && !g.is_null() {
                    return true;
                }
            }
            sleep_ms(500).await;
        }
        false
    }

    async fn sleep_ms(ms: i32) {
        let promise = js_sys::Promise::new(&mut |resolve, _reject| {
            if let Some(w) = web_sys::window() {
                let _ = w.set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, ms);
            }
        });
        let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
    }

    /// LIVE PROBE A — receiver pipe (uwuwu#1454 verification): `captureException`
    /// against the real SDK the production loader URL serves. Compiled with
    /// SENTRY_DSN/SENTRY_ENVIRONMENT baked in. `#[ignore]`-gated: an
    /// early-returning test would be permanently green without asserting
    /// anything (uwuwu#1454 review) — run explicitly, in a probe build only,
    /// via `wasm-bindgen-test-runner <wasm> -- --ignored` with
    /// SENTRY_ENVIRONMENT=probe (delivery is then verified by the event
    /// landing in the probe environment, not by the exit code).
    #[wasm_bindgen_test]
    #[ignore = "live delivery probe — probe build only, run with --ignored"]
    async fn live_probe_receiver_pipe() {
        if ENVIRONMENT != "probe" || DSN.is_empty() {
            return;
        }
        init_with(DSN, RELEASE, ENVIRONMENT);
        assert!(wait_sentry(40).await, "SDK did not load within 20s");

        // Direct receiver call — the exact surface the tracing layer uses.
        let g = sentry_global().expect("window.Sentry");
        let err = js_sys::Error::new("probe-a uwuwu1454 receiver pipe");
        let cap = Reflect::get(&g, &"captureException".into()).expect("captureException export");
        let cap_fn = cap
            .dyn_ref::<js_sys::Function>()
            .expect("captureException is a function");
        let _ = cap_fn.call1(&g, &err);

        // The layer fires flush(2000) internally; give the HTTP round-trip
        // its bounded window before the test page dies.
        sleep_ms(3000).await;
    }

    /// LIVE PROBE C — the revived error! channel end-to-end
    /// (uwuwu#1454 verification): a real `tracing::error!` goes through the
    /// `tracing_console` layer → console.error + captureException → Sentry.
    /// `#[ignore]`-gated (see probe A).
    #[wasm_bindgen_test]
    #[ignore = "live delivery probe — probe build only, run with --ignored"]
    async fn live_probe_error_channel() {
        if ENVIRONMENT != "probe" || DSN.is_empty() {
            return;
        }
        crate::init_tracing();
        assert!(wait_sentry(40).await, "SDK did not load within 20s");

        // The layer routes this to console.error + captureException.
        tracing::error!("probe-c uwuwu1454 error channel end-to-end");

        sleep_ms(3000).await;
    }
}
