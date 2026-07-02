//! `origa_ui` build-time configuration: TrailBase URL resolution.
//!
//! Pure (no env access) so it can be unit-tested via `#[path]` from
//! `origa_ui/tests/build_config.rs`, mirroring the `tauri`-side pattern
//! (`tauri/build_config.rs` + `tauri/tests/build_config.rs`). All environment
//! access lives in `build.rs`.

#[path = "../build_defaults.rs"]
mod defaults;

/// Resolve the TrailBase URL to emit via `cargo:rustc-env`. Falls back to the
/// production default when the env var is unset OR empty.
///
/// Empty handling is deliberate. `env!()` panics only when an env var is UNSET;
/// for a var SET to an empty string it silently returns `""`. That empty value
/// produced two bugs: (1) a relative fetch URL the WebView resolved against its
/// own origin (`tauri.localhost`) in `trailbase_client.rs:43`, and (2) JWT
/// issuer validation in `trailbase_auth.rs:63` where `iss == ""` never matched
/// the real issuer, spamming `tracing::warn!("Untrusted JWT issuer: ...")`.
/// Treating empty as "use default" closes both holes.
///
/// Note `cargo:rustc-env` UNCONDITIONALLY sets the variable for the rustc
/// invocation (The Cargo Book, "Outputs of the Build Script") — a different
/// mechanism from `.cargo/config.toml [env]`, which needs `force = true` to
/// override. So the emitted value overrides even an empty host value, and the
/// fix is crate-wide: both `env!("TRAILBASE_URL")` sites resolve to it. See
/// ADR-020.
pub(crate) fn resolve_trailbase(env_value: Option<&str>) -> String {
    match env_value {
        Some(v) if !v.is_empty() => v.to_string(),
        _ => defaults::DEFAULT_TRAILBASE.to_string(),
    }
}
