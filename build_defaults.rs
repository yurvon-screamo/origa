//! Shared build-time production default for the TrailBase backend host.
//!
//! Single source of truth for `DEFAULT_TRAILBASE`, `#[path]`-included by both
//! `tauri/build_config.rs` and `origa_ui/build_config.rs`. Build scripts cannot
//! `use` workspace crates — they compile with `std` + `[build-dependencies]`
//! only — so the existing `#[path]`-include pattern (see `tauri/build.rs:20-21`)
//! is reused instead of introducing a workspace crate.
//!
//! Only `DEFAULT_TRAILBASE` lives here because it is the only default needed by
//! BOTH crates. `DEFAULT_CDN` and `DEFAULT_LANDING` are `tauri`-only (the
//! `origa_ui` build script uses a strict `env!()` for the CDN with no fallback
//! and never references the landing host), so moving them here would create
//! unused constants inside the `origa_ui` build binary — and the project
//! forbids `#[allow(dead_code)]` (see `AGENTS.md`). See ADR-020 for the full
//! rationale.

/// Production TrailBase URL. Used as the fallback when the `TRAILBASE_URL` env
/// var is unset OR empty (an empty value previously produced a relative fetch
/// URL the WebView resolved against its own origin — see ADR-020).
pub(crate) const DEFAULT_TRAILBASE: &str = "https://app.origa.uwuwu.net";
