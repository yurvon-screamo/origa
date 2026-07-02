//! `origa_landing` build-time configuration: landing base URL resolution.
//!
//! Pure (no env access) so it can be unit-tested via `#[path]` from
//! `origa_landing/tests/build_config.rs`, mirroring the `tauri` / `origa_ui`
//! pattern (`<crate>/build_config.rs` + `<crate>/tests/build_config.rs`). All
//! environment access lives in `build.rs`.
//!
//! `DEFAULT_LANDING` is local to this crate rather than in the shared root
//! `build_defaults.rs`. That shared file is `#[path]`-included by `origa_ui`,
//! which never references the landing host — a shared `DEFAULT_LANDING` would
//! be dead code in the `origa_ui` build binary, and the project forbids
//! `#[allow(dead_code)]` (`AGENTS.md`). The same value also lives in
//! `tauri/build_config.rs` (used for CSP injection); both are pinned to the
//! canonical `https://origa.uwuwu.net` and guarded by drift-detection tests.
//! See ADR-024 for the generalized principle and this constraint.

/// Production landing base URL. Emitted via `cargo:rustc-env` and consumed at
/// compile time by `env!("ORIGA_LANDING_BASE_URL")` in `seo.rs:6`, which forms
/// every canonical URL, Open Graph tag, and JSON-LD `url`/`logo`. An empty
/// value would make all of those relative — an SEO catastrophe — so empty is
/// treated as "use default" rather than passed through.
pub(crate) const DEFAULT_LANDING: &str = "https://origa.uwuwu.net";

/// Resolve a build-script env var to a non-empty value, falling back to
/// `default` when the var is unset OR set to an empty string.
///
/// `env::var(...).unwrap_or_else(|_| default)` catches only the UNSET case
/// (the `Err`); for a var SET to `""` it returns `Ok("")` and the fallback is
/// skipped, yielding an empty value downstream. Treating empty as "use
/// default" closes that hole. See ADR-024 for the generalized principle.
pub(crate) fn resolve_env(env_value: Option<&str>, default: &str) -> String {
    match env_value {
        Some(v) if !v.is_empty() => v.to_string(),
        _ => default.to_string(),
    }
}
