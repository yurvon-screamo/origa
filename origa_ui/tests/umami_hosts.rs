//! Cross-crate drift guard for the analytics endpoints (ADR-054 §7).
//!
//! The 2026-09 incident (ADR-054 §3) was a silent beacon loss caused by two
//! files disagreeing about a host: the desktop CSP pinned the tracker's
//! script origin while the tracker actually sent beacons to a third host.
//! Nothing in the test suite tied those strings together — the byte-equality
//! test only compared `build_csp` with the committed `tauri.conf.json`, and
//! the wasm test only checked the injected tag's attributes.
//!
//! This guard pins the four surfaces that must name the same analytics
//! endpoints:
//!
//! | Surface | File | Values |
//! |---|---|---|
//! | WASM injection | `origa_ui/src/core/analytics.rs` | `UMAMI_SCRIPT_SRC`, `UMAMI_HOST_URL` |
//! | Landing tag | `origa_landing/src/app.rs` | `src`, `data-host-url`, `data-website-id` |
//! | Desktop CSP (built) | `tauri/build_config.rs` | `script-src`, `connect-src` |
//! | Desktop CSP (committed) | `tauri/tauri.conf.json` | `csp` |
//!
//! plus the website id, which the landing tag and `origa_ui/build_config.rs`
//! must share (one website for all surfaces — ADR-054 §Decision-1).

const ANALYTICS_RS: &str = include_str!("../src/core/analytics.rs");
const BUILD_CONFIG_RS: &str = include_str!("../build_config.rs");
const LANDING_APP_RS: &str = include_str!("../../origa_landing/src/app.rs");
const TAURI_BUILD_CONFIG_RS: &str = include_str!("../../tauri/build_config.rs");
const TAURI_CONF: &str = include_str!("../../tauri/tauri.conf.json");

/// Hosts of the retired Umami Cloud deployment. Their presence in any of the
/// surfaces above means a half-finished migration: beacons would be sent to a
/// host the CSP does not allow (or vice versa) — the exact silent failure of
/// ADR-054 §3.
const RETIRED_HOSTS: [&str; 2] = ["cloud.umami.is", "gateway.umami.is"];

/// Value of the double-quoted string that follows `needle`.
fn value_after(source: &str, needle: &str) -> String {
    let start = source
        .find(needle)
        .unwrap_or_else(|| panic!("{needle:?} not found"));
    let rest = &source[start + needle.len()..];
    let end = rest
        .find('"')
        .unwrap_or_else(|| panic!("unterminated string after {needle:?}"));
    rest[..end].to_string()
}

/// The CSP literal of the `tauri` build config (the `format!` template with
/// the `{placeholder}` slots) — not the whole file, whose doc comments also
/// mention the analytics host.
fn built_csp_template() -> String {
    value_after(TAURI_BUILD_CONFIG_RS, "\"default-src 'self'")
}

/// The CSP literal committed in `tauri.conf.json`.
fn committed_csp() -> String {
    value_after(TAURI_CONF, "\"csp\": \"")
}

#[test]
fn tracker_script_and_collection_endpoint_share_one_host() {
    let script_src = value_after(ANALYTICS_RS, "const UMAMI_SCRIPT_SRC: &str = \"");
    let host_url = value_after(ANALYTICS_RS, "const UMAMI_HOST_URL: &str = \"");

    assert_eq!(
        script_src,
        format!("{host_url}/script.js"),
        "the tracker script must be served from the pinned collection host"
    );
}

#[test]
fn landing_tag_matches_the_wasm_injection() {
    let script_src = value_after(ANALYTICS_RS, "const UMAMI_SCRIPT_SRC: &str = \"");
    let host_url = value_after(ANALYTICS_RS, "const UMAMI_HOST_URL: &str = \"");

    assert_eq!(value_after(LANDING_APP_RS, "src=\""), script_src);
    assert_eq!(value_after(LANDING_APP_RS, "data-host-url=\""), host_url);
}

#[test]
fn landing_and_app_report_to_one_website() {
    let default_id = value_after(BUILD_CONFIG_RS, "const DEFAULT_UMAMI_WEBSITE_ID: &str = \"");
    assert_eq!(
        value_after(LANDING_APP_RS, "data-website-id=\""),
        default_id
    );
}

#[test]
fn desktop_csp_allows_the_analytics_host_in_script_and_connect() {
    let host_url = value_after(ANALYTICS_RS, "const UMAMI_HOST_URL: &str = \"");

    for (name, csp) in [
        ("built", built_csp_template()),
        ("committed", committed_csp()),
    ] {
        let occurrences = csp.matches(&host_url).count();
        assert_eq!(
            occurrences, 2,
            "{name} CSP must allow {host_url} exactly twice \
             (script-src for the tracker, connect-src for the beacons); got {occurrences}"
        );
    }
}

#[test]
fn no_surface_still_references_the_retired_cloud_hosts() {
    // Only the functional values are checked — the source files legitimately
    // mention the retired hosts in their doc comments (the ADR-054
    // post-mortem explains what went wrong with them).
    let functional_values = [
        (
            "origa_ui UMAMI_SCRIPT_SRC",
            value_after(ANALYTICS_RS, "const UMAMI_SCRIPT_SRC: &str = \""),
        ),
        (
            "origa_ui UMAMI_HOST_URL",
            value_after(ANALYTICS_RS, "const UMAMI_HOST_URL: &str = \""),
        ),
        ("landing script src", value_after(LANDING_APP_RS, "src=\"")),
        (
            "landing data-host-url",
            value_after(LANDING_APP_RS, "data-host-url=\""),
        ),
        ("tauri built CSP", built_csp_template()),
        ("tauri committed CSP", committed_csp()),
    ];

    for (name, value) in functional_values {
        for host in RETIRED_HOSTS {
            assert!(
                !value.contains(host),
                "{name} still references the retired host {host}: {value}"
            );
        }
    }
}
