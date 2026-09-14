//! Drift-detection and Prove-It tests for `origa_ui/build_config.rs`.
//!
//! Strategy: `#[path]`-include the pure `build_config` module and exercise
//! `resolve_trailbase` / `resolve_cdn` across the three input shapes (unset /
//! empty / set). The "unset" cases also serve as drift guards for
//! `DEFAULT_TRAILBASE` / `DEFAULT_CDN`: the fallback branch returns exactly
//! that constant, so any change to its value breaks the test.

#[path = "../build_config.rs"]
mod build_config;

use build_config::{DEFAULT_UMAMI_WEBSITE_ID, resolve_cdn, resolve_trailbase, resolve_umami};

#[test]
fn trailbase_uses_production_default_when_unset() {
    assert_eq!(resolve_trailbase(None), "https://app.origa.uwuwu.net");
}

#[test]
fn trailbase_uses_production_default_when_empty() {
    assert_eq!(resolve_trailbase(Some("")), "https://app.origa.uwuwu.net");
}

#[test]
fn trailbase_uses_explicit_value_when_set() {
    assert_eq!(
        resolve_trailbase(Some("https://staging.example.com")),
        "https://staging.example.com"
    );
}

#[test]
fn cdn_uses_production_default_when_unset() {
    assert_eq!(resolve_cdn(None), "https://s3.origa.uwuwu.net");
}

#[test]
fn cdn_uses_production_default_when_empty() {
    assert_eq!(resolve_cdn(Some("")), "https://s3.origa.uwuwu.net");
}

#[test]
fn cdn_uses_explicit_value_when_set() {
    assert_eq!(
        resolve_cdn(Some("https://cdn.staging.example.com")),
        "https://cdn.staging.example.com"
    );
}

#[test]
fn umami_uses_production_default_when_unset() {
    assert_eq!(resolve_umami(None, None), DEFAULT_UMAMI_WEBSITE_ID);
}

#[test]
fn umami_uses_production_default_when_website_id_empty() {
    assert_eq!(resolve_umami(None, Some("")), DEFAULT_UMAMI_WEBSITE_ID);
}

#[test]
fn umami_uses_explicit_website_id_when_set() {
    assert_eq!(
        resolve_umami(None, Some("11111111-2222-3333-4444-555555555555")),
        "11111111-2222-3333-4444-555555555555"
    );
}

#[test]
fn umami_disabled_one_disables_analytics() {
    assert_eq!(resolve_umami(Some("1"), None), "");
}

#[test]
fn umami_disabled_true_lowercase_disables_analytics() {
    assert_eq!(resolve_umami(Some("true"), None), "");
}

#[test]
fn umami_disabled_true_mixed_case_disables_analytics() {
    assert_eq!(resolve_umami(Some("TRUE"), None), "");
    assert_eq!(resolve_umami(Some("True"), None), "");
}

#[test]
fn umami_disabled_zero_keeps_analytics_enabled() {
    assert_eq!(resolve_umami(Some("0"), None), DEFAULT_UMAMI_WEBSITE_ID);
}

#[test]
fn umami_disabled_false_keeps_analytics_enabled() {
    assert_eq!(resolve_umami(Some("false"), None), DEFAULT_UMAMI_WEBSITE_ID);
}

#[test]
fn umami_disabled_empty_keeps_analytics_enabled() {
    assert_eq!(resolve_umami(Some(""), None), DEFAULT_UMAMI_WEBSITE_ID);
}

#[test]
fn umami_disabled_flag_wins_over_explicit_website_id() {
    assert_eq!(
        resolve_umami(Some("1"), Some("11111111-2222-3333-4444-555555555555")),
        ""
    );
}
