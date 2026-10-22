//! CSS rebuild decision for `origa_landing/build.rs` (issue #832).
//!
//! Pure (no filesystem/env access) so it can be unit-tested via `#[path]`
//! from `origa_landing/tests/css_rebuild.rs`, mirroring the `build_config.rs`
//! pattern (`<crate>/build_config.rs` + `<crate>/tests/build_config.rs`). All
//! I/O lives in `build.rs`.
//!
//! Background: a fresh clone has no `origa_landing/node_modules`, and the
//! production Docker image has no node at all — in both cases the committed
//! `style/landing.processed.css` is the stylesheet. Shelling out to
//! `npx tailwindcss` unconditionally and panicking on its non-zero exit
//! turned every such build red (issue #832).

/// What `build_css` should do about the Tailwind stylesheet.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum CssAction {
    /// Tailwind is installed — shell out to `npx tailwindcss` and treat a
    /// non-zero exit as a genuine build error (bad `tailwind.config.js`,
    /// broken input CSS, damaged `node_modules`).
    Rebuild,
    /// Tailwind is not installed, but a stylesheet file exists — warn and
    /// keep building with it. This is how the production Docker image always
    /// worked, so it is the documented fallback, not an error.
    UseExisting,
    /// Tailwind is not installed and no stylesheet exists — the binary would
    /// serve 404 on `/landing.processed.css`. `build_css` fails the build
    /// with install instructions after the decision.
    Fail,
}

/// Decide the CSS rebuild action from two filesystem facts.
///
/// - `tailwind_installed`: `node_modules/tailwindcss/package.json` exists.
///   The package directory is checked instead of a `.bin` shim because npm
///   and bun both create it and it is identical across platforms.
/// - `css_output_exists`: `style/landing.processed.css` exists (committed or
///   locally built).
pub(crate) fn css_rebuild_action(tailwind_installed: bool, css_output_exists: bool) -> CssAction {
    if tailwind_installed {
        CssAction::Rebuild
    } else if css_output_exists {
        CssAction::UseExisting
    } else {
        CssAction::Fail
    }
}
