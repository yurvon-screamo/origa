//! Build-time configuration parameterization.
//!
//! Single Rust-side source for the production hosts used by `tauri/build.rs`
//! to construct the CSP that is injected into Tauri's config via the
//! `TAURI_CONFIG` env var (RFC 7396 JSON Merge Patch). See ADR-009 for the
//! full rationale, including the residual sync points that make this a
//! "reduced duplication" rather than a true single source of truth.
//!
//! This module is pure (no I/O, no env access) so that it can be unit-tested
//! via `#[path]` from `tauri/tests/build_config.rs`. All env var resolution
//! lives in `build.rs`.
//!
//! `tauri/capabilities/default.json` is NOT parameterized here — build scripts
//! must not mutate committed source files. Its opener allow-list is validated
//! against a separate template living in `tauri/tests/build_config.rs`.

/// Production CDN base URL. Used when `ORIGA_CDN_BASE_URL` env var is unset
/// (e.g., CI builds without env propagation — see ADR-009 "CI constraint").
pub(crate) const DEFAULT_CDN: &str = "https://s3-proxy-production-52e3.up.railway.app";

/// Production TrailBase URL. Used when `TRAILBASE_URL` env var is unset.
pub(crate) const DEFAULT_TRAILBASE: &str = "https://app.origa.uwuwu.net";

/// Production landing URL. Used when `ORIGA_LANDING_BASE_URL` env var is unset.
pub(crate) const DEFAULT_LANDING: &str = "https://origa.uwuwu.net";

/// Build the Content-Security-Policy directive by substituting env-controlled
/// hosts into the static template. Third-party hosts (huggingface, Google Fonts,
/// OAuth providers) remain hardcoded — they are not environment-dependent.
///
/// The literal is kept on a single line because `rustfmt` does not reflow
/// string-literal contents, and byte-equality with `tauri.conf.json` must hold
/// (verified by `build_csp_with_production_defaults_matches_committed_tauri_conf`).
pub(crate) fn build_csp(cdn: &str, landing: &str, trailbase: &str) -> String {
    format!(
        "default-src 'self'; script-src 'self' 'unsafe-inline' 'unsafe-eval'; connect-src 'self' ipc: http://ipc.localhost {cdn} {landing} {trailbase} https://huggingface.co; img-src 'self' data: blob: {cdn}; media-src 'self' blob: {cdn}; style-src 'self' 'unsafe-inline' https://fonts.googleapis.com; font-src 'self' https://fonts.gstatic.com; form-action 'self' https://accounts.google.com https://oauth.yandex.ru; frame-ancestors 'none'"
    )
}

/// Apply a RFC 7396 JSON Merge Patch to `target` in place.
///
/// serde_json (1.0.150 at time of writing) does NOT expose a public `merge`
/// API on `Value`, so this is a minimal local implementation of RFC 7396 §2
/// ("MergePatch"). Lives in this pure module so that it can be unit-tested via
/// `#[path]` from `tauri/tests/build_config.rs` (build scripts themselves are
/// not covered by `cargo test`).
///
/// Semantics (RFC 7396 §2). If `patch` is not an Object, it replaces `target`
/// entirely. If `patch` is an Object, `target` is forced to an Object (a
/// non-object target becomes empty), then for each `(key, value)` in `patch`:
/// when `value == null`, `key` is removed from `target` (no-op if absent);
/// otherwise `target[key]` is recursively merged with `value`. A missing
/// `target[key]` is treated as `null`, which the recursive call promotes to an
/// empty object before merging (per RFC 7396).
///
/// Recursion depth is bounded by the nesting depth of `patch`. For the inputs
/// used here (a flat Tauri CLI merge patch + a 3-level-deep CSP patch) this is
/// trivially stack-safe.
pub(crate) fn apply_merge_patch(target: &mut serde_json::Value, patch: serde_json::Value) {
    use serde_json::{Map, Value};

    let patch_obj = match patch {
        Value::Object(map) => map,
        // Non-object patch replaces the target entirely (RFC 7396 §2).
        other => {
            *target = other;
            return;
        },
    };

    // Force target to be an Object so we can merge into it (RFC 7396 §2: a
    // non-object target is replaced by an empty object when the patch is an
    // object).
    if !target.is_object() {
        *target = Value::Object(Map::new());
    }
    let target_obj = target
        .as_object_mut()
        .expect("target is Object (forced above; invariant cannot be violated)");

    for (key, value) in patch_obj {
        if value.is_null() {
            // RFC 7396: a null value in the patch deletes the key.
            target_obj.remove(&key);
        } else {
            // `entry().or_insert(Value::Null)` yields an existing slot or
            // inserts Null for a missing key. The recursive call then merges
            // `value` into it. RFC 7396 treats a missing target key as an
            // implicit empty object, which the recursion materializes correctly
            // (non-object target → empty object on the next call).
            let slot = target_obj.entry(key).or_insert(Value::Null);
            apply_merge_patch(slot, value);
        }
    }
}
