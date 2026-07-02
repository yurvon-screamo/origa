//! Prove-It and drift-detection tests for `origa_landing/build_config.rs`.
//!
//! `#[path]`-includes the pure `build_config` module and exercises
//! `resolve_env` across the three input shapes (unset / empty / set). The
//! unset case asserts the literal canonical landing host, so it doubles as a
//! drift guard for `DEFAULT_LANDING` (a revert to the stale `origa.app`
//! breaks it).

#[path = "../build_config.rs"]
mod build_config;

use build_config::{DEFAULT_LANDING, resolve_env};

#[test]
fn resolve_env_uses_default_when_unset() {
    assert_eq!(
        resolve_env(None, DEFAULT_LANDING),
        "https://origa.uwuwu.net"
    );
}

/// The empty-shell-var bug case: a var SET to `""` must still fall back to the
/// default. A naive `unwrap_or_else` would return `""` here (it only catches
/// the `Err` of an unset var), producing relative canonical/OG/JSON-LD URLs.
#[test]
fn resolve_env_uses_default_when_empty() {
    assert_eq!(
        resolve_env(Some(""), DEFAULT_LANDING),
        "https://origa.uwuwu.net"
    );
}

#[test]
fn resolve_env_uses_explicit_value_when_set() {
    assert_eq!(
        resolve_env(Some("https://staging.example.com"), DEFAULT_LANDING),
        "https://staging.example.com"
    );
}
