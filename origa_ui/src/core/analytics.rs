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
/// No-op when analytics was compiled out (`UMAMI_WEBSITE_ID` empty) or the
/// DOM is unavailable. Injection failure is non-fatal by design — losing
/// analytics must never break the app — so errors are logged, not propagated.
///
/// Note: `defer` is intentionally NOT set — it only affects parser-inserted
/// tags, and this script is inserted at runtime. Execution order is
/// irrelevant for the tracker (it reports the current URL when it runs).
pub(crate) fn inject_umami() {
    let website_id = env!("UMAMI_WEBSITE_ID");
    if website_id.is_empty() {
        return;
    }

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
