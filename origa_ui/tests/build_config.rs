//! Drift-detection and Prove-It tests for `origa_ui/build_config.rs`.
//!
//! Strategy: `#[path]`-include the pure `build_config` module and exercise
//! `resolve_trailbase` / `resolve_cdn` across the three input shapes (unset /
//! empty / set). The "unset" cases also serve as drift guards for
//! `DEFAULT_TRAILBASE` / `DEFAULT_CDN`: the fallback branch returns exactly
//! that constant, so any change to its value breaks the test.

#[path = "../build_config.rs"]
mod build_config;

use build_config::{
    DEFAULT_UMAMI_DEV_WEBSITE_ID, DEFAULT_UMAMI_WEBSITE_ID, resolve_cdn, resolve_trailbase,
    resolve_umami, umami_default_website_id,
};

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

/// CI-version sample standing in for a real `ORIGA_VERSION` value.
const CI_VERSION: &str = "1.2.3";

#[test]
fn umami_local_build_defaults_to_dev_website() {
    // No ORIGA_VERSION at all (trunk serve, cargo tauri dev, local e2e, shots).
    assert_eq!(
        resolve_umami(None, None, None),
        DEFAULT_UMAMI_DEV_WEBSITE_ID
    );
}

#[test]
fn umami_empty_version_defaults_to_dev_website() {
    // ADR-020 empty==unset contract: Some("") must not fall into the
    // production branch (review Common: a silent prod hit would be the
    // opposite of the isolation goal).
    assert_eq!(
        resolve_umami(None, None, Some("")),
        DEFAULT_UMAMI_DEV_WEBSITE_ID
    );
}

#[test]
fn umami_literal_dev_version_defaults_to_dev_website() {
    // build.rs's own unwrap_or("dev") default for local builds.
    assert_eq!(
        resolve_umami(None, None, Some("dev")),
        DEFAULT_UMAMI_DEV_WEBSITE_ID
    );
}

#[test]
fn umami_real_version_defaults_to_production_website() {
    assert_eq!(
        resolve_umami(None, None, Some(CI_VERSION)),
        DEFAULT_UMAMI_WEBSITE_ID
    );
}

#[test]
fn umami_default_website_id_classifies_by_context() {
    assert_eq!(umami_default_website_id(None), DEFAULT_UMAMI_DEV_WEBSITE_ID);
    assert_eq!(
        umami_default_website_id(Some("")),
        DEFAULT_UMAMI_DEV_WEBSITE_ID
    );
    assert_eq!(
        umami_default_website_id(Some("dev")),
        DEFAULT_UMAMI_DEV_WEBSITE_ID
    );
    assert_eq!(
        umami_default_website_id(Some(CI_VERSION)),
        DEFAULT_UMAMI_WEBSITE_ID
    );
    // Prerelease/rc shapes are real CI versions, not the "dev" sentinel.
    assert_eq!(
        umami_default_website_id(Some("0.7.0-rc1")),
        DEFAULT_UMAMI_WEBSITE_ID
    );
}

#[test]
fn umami_uses_explicit_website_id_when_set() {
    // Explicit override wins over the context default in both directions.
    assert_eq!(
        resolve_umami(
            None,
            Some("11111111-2222-3333-4444-555555555555"),
            Some(CI_VERSION)
        ),
        "11111111-2222-3333-4444-555555555555"
    );
    assert_eq!(
        resolve_umami(None, Some("11111111-2222-3333-4444-555555555555"), None),
        "11111111-2222-3333-4444-555555555555"
    );
}

#[test]
fn umami_explicit_empty_website_id_falls_back_to_context_default() {
    // resolve_trailbase/resolve_cdn empty-handling contract (ADR-020):
    // empty override means "not set", not "disable the default".
    assert_eq!(
        resolve_umami(None, Some(""), Some(CI_VERSION)),
        DEFAULT_UMAMI_WEBSITE_ID
    );
    assert_eq!(
        resolve_umami(None, Some(""), None),
        DEFAULT_UMAMI_DEV_WEBSITE_ID
    );
}

#[test]
fn umami_disabled_one_disables_analytics() {
    assert_eq!(resolve_umami(Some("1"), None, Some(CI_VERSION)), "");
}

#[test]
fn umami_disabled_true_lowercase_disables_analytics() {
    assert_eq!(resolve_umami(Some("true"), None, Some(CI_VERSION)), "");
}

#[test]
fn umami_disabled_true_mixed_case_disables_analytics() {
    assert_eq!(resolve_umami(Some("TRUE"), None, Some(CI_VERSION)), "");
    assert_eq!(resolve_umami(Some("True"), None, Some(CI_VERSION)), "");
}

#[test]
fn umami_disabled_zero_keeps_analytics_enabled() {
    // Version arg present here too: the mute flag must not depend on it.
    assert_eq!(
        resolve_umami(Some("0"), None, Some(CI_VERSION)),
        DEFAULT_UMAMI_WEBSITE_ID
    );
}

#[test]
fn umami_disabled_false_keeps_analytics_enabled() {
    assert_eq!(
        resolve_umami(Some("false"), None, Some(CI_VERSION)),
        DEFAULT_UMAMI_WEBSITE_ID
    );
}

#[test]
fn umami_disabled_empty_keeps_analytics_enabled() {
    assert_eq!(
        resolve_umami(Some(""), None, Some(CI_VERSION)),
        DEFAULT_UMAMI_WEBSITE_ID
    );
}

#[test]
fn umami_disabled_flag_wins_over_explicit_website_id() {
    assert_eq!(
        resolve_umami(
            Some("1"),
            Some("11111111-2222-3333-4444-555555555555"),
            Some(CI_VERSION)
        ),
        ""
    );
}

#[test]
fn umami_disabled_flag_wins_over_context_default() {
    // CI e2e/android-smoke stay fully muted even though they build with
    // ORIGA_VERSION-free or dev-tagged contexts.
    assert_eq!(resolve_umami(Some("1"), None, None), "");
    assert_eq!(resolve_umami(Some("true"), None, Some("dev")), "");
}

#[test]
fn umami_drift_guards_pin_both_website_ids() {
    // Change-of-value guards: the fallback branch returns exactly the
    // constants, so any edit to their values breaks these asserts.
    assert_eq!(
        umami_default_website_id(Some(CI_VERSION)),
        "57c7d57f-3bd9-48cf-b9ee-ea02c95d967a"
    );
    assert_eq!(
        umami_default_website_id(None),
        "f0af05d8-a31b-462c-8f6c-999b95da1aa4"
    );
}
