//! `origa_ui` build-time configuration: TrailBase and CDN URL resolution.
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

/// Resolve the CDN base URL to emit via `cargo:rustc-env`. Falls back to the
/// production default when the env var is unset OR empty.
///
/// Empty handling is deliberate — same class of bug as `resolve_trailbase`
/// (ADR-020). `env!()` panics only when an env var is UNSET; for a var SET to
/// an empty string it silently returns `""`. That empty value flowed into
/// `cdn_url()` (`config.rs:57`) and `build_manifest_url()`
/// (`cache_manager.rs:89`), producing relative URLs (`/grammar/grammar_v3.json`)
/// that the WebView resolved against its own origin (`tauri.localhost`)
/// instead of the CDN, causing `CacheFirstCdnProvider` to wrap locally-served
/// resources in blob URLs. See ADR-023.
pub(crate) fn resolve_cdn(env_value: Option<&str>) -> String {
    match env_value {
        Some(v) if !v.is_empty() => v.to_string(),
        _ => defaults::DEFAULT_CDN.to_string(),
    }
}

/// Production website ID on the self-hosted Umami instance (umami.origa.uwuwu.net,
/// website «Origa»). Website IDs are public identifiers — every visitor's page
/// HTML exposes them in the tracker tag — so committing the default is safe
/// (unlike, say, an API key). Used by `resolve_umami` as the default for
/// production-context builds, mirroring the `resolve_trailbase` empty-handling
/// contract (ADR-020). See ADR-054 (integration) and §7 (self-host migration,
/// 2026-10).
pub(crate) const DEFAULT_UMAMI_WEBSITE_ID: &str = "57c7d57f-3bd9-48cf-b9ee-ea02c95d967a";

/// Dev/test website ID on the same Umami instance (website «Origa (dev)»,
/// domain `localhost`). Local builds — `trunk serve`, `cargo tauri dev`,
/// local e2e (`playwright.config.ts` webServer) and local shots — all run
/// without `ORIGA_VERSION`, so `umami_default_website_id` routes their
/// localhost/tauri.localhost traffic here instead of polluting the
/// production dashboard (isolation task 20261109071819; ADR-054 consequence
/// «splitting into two websites remains a plan upgrade away»). Same public
/// identifier status as the production ID above.
pub(crate) const DEFAULT_UMAMI_DEV_WEBSITE_ID: &str = "f0af05d8-a31b-462c-8f6c-999b95da1aa4";

/// Website-id default by build context. `ORIGA_VERSION` is set on CI
/// surfaces (`docker.yml`, `tauri.yml`); `build.rs` reads it through
/// `env::var`, so unset arrives as `None` and an empty value as `Some("")`.
/// Per the ADR-020 empty==unset contract both fall to the dev website, as
/// does the literal `"dev"` — shipped as `ENV ORIGA_VERSION=dev` in
/// `origa_ui/Dockerfile` (local docker builds) and settable by a developer
/// shell. Any other non-empty value is a real CI version → production
/// website.
pub(crate) fn umami_default_website_id(version: Option<&str>) -> &'static str {
    match version {
        Some(v) if !v.is_empty() && v != "dev" => DEFAULT_UMAMI_WEBSITE_ID,
        _ => DEFAULT_UMAMI_DEV_WEBSITE_ID,
    }
}

/// Resolve the Umami website ID to emit via `cargo:rustc-env`. Returns
/// an empty string (= analytics compiled out, runtime injection is a no-op)
/// when `disabled` is truthy; otherwise falls back from an explicit
/// non-empty `website_id` to the context default: production website for
/// builds with a real `ORIGA_VERSION`, dev website for local builds
/// ([`umami_default_website_id`]).
///
/// `UMAMI_DISABLED` exists so CI builds that actually execute the app (e2e
/// headless runs, the android-smoke emulator) can keep their
/// localhost/tauri.localhost traffic out of analytics entirely — a static
/// tracker tag in `index.html` cannot be muted per build, which is why the
/// tag is injected from WASM instead (ADR-054). The context default covers
/// the complementary leak: CI already mutes, but local dev/e2e/shots runs
/// were landing in the production dashboard through the `localhost` entry
/// of the tracker's `data-domains` allow-list (that entry is load-bearing
/// for desktop users — Linux/macOS Tauri normalize to hostname `localhost`).
pub(crate) fn resolve_umami(
    disabled: Option<&str>,
    website_id: Option<&str>,
    version: Option<&str>,
) -> String {
    if is_truthy(disabled) {
        return String::new();
    }
    match website_id {
        Some(v) if !v.is_empty() => v.to_string(),
        _ => umami_default_website_id(version).to_string(),
    }
}

/// Truthy values for `UMAMI_DISABLED`: exactly `"1"` or `"true"`, matched
/// case-insensitively. Anything else (`"0"`, `"false"`, `""`, arbitrary
/// strings, unset) leaves analytics enabled — fail-open, because a typo'd
/// flag value silently losing analytics on a release build is worse than a
/// noisy CI run.
fn is_truthy(value: Option<&str>) -> bool {
    matches!(value, Some(v) if v.eq_ignore_ascii_case("1") || v.eq_ignore_ascii_case("true"))
}
