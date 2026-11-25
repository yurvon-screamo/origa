//! Structural guard: `register_all()` must be Windows-only.
//!
//! Owner decision in uwuwu#1098 (variant В, sanctioned 24.11.2026): on Linux
//! the runtime scheme registration is retired. `tauri-plugin-deep-link`
//! `register_all()` rewrites the system-wide `origa://` scheme default to the
//! *running* binary's absolute path on every launch ("last run wins"), which
//! destabilizes the handler on dev-active machines: debug builds from task
//! worktrees overwrite it, the factory removes the worktree afterwards →
//! dead handler → the OAuth browser callback dies outside the webview → the
//! user stays on the login page. Every distribution channel (deb/rpm/AUR)
//! registers the scheme statically via the bundled `Origa.desktop`
//! (`MimeType=x-scheme-handler/origa`), so the runtime path adds nothing for
//! packaged installs; retiring it also removes the
//! `update-desktop-database`-not-found error noise (Sentry RUST-Z/RUST-F,
//! uwuwu#1511).
//!
//! Positional check — find the registration call and inspect the cfg
//! attribute immediately above its block, so the guard survives
//! indentation/reflow churn (same pattern as the store-build guards in
//! build_config.rs).

#[test]
fn register_all_is_windows_only() {
    let lib_rs = include_str!("../src/lib.rs");

    let anchor = lib_rs
        .find("app.deep_link().register_all()")
        .expect("register_all() call missing from lib.rs — the deep-link gate moved?");

    // The cfg attribute must sit immediately above the gated block: a sanity
    // window keeps the failure message useful if the code between the
    // attribute and the call ever grows a nested `#[cfg(`.
    let window = &lib_rs[..anchor];
    let cfg_start = window
        .rfind("#[cfg(")
        .filter(|start| anchor - start < 512)
        .expect("cfg attribute above register_all() missing or too far away");

    let cfg_attr = &lib_rs[cfg_start..anchor];

    assert!(
        cfg_attr.starts_with("#[cfg(windows)]"),
        "register_all() must be gated #[cfg(windows)] — on Linux the runtime \
         scheme registration is retired (uwuwu#1098 variant В, \
         owner-sanctioned 24.11.2026): last-run-wins rewrote the system \
         origa:// default with the running binary's path, dead-ending the \
         OAuth callback (login-page strand). Got: {cfg_attr}"
    );
    assert!(
        !cfg_attr.contains("linux"),
        "register_all() must not run on Linux — static package registration \
         (Origa.desktop MimeType) covers every Linux channel. Got: {cfg_attr}"
    );
}
