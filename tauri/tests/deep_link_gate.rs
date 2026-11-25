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

/// Regression pin for the tester's live finding (uwuwu#1098). GLib drops the
/// URI when a desktop entry's `Exec` has no `%u` field code: with the packaged
/// `Origa.desktop` (`Exec=origa-app`, rc40 .deb verified) as the scheme
/// default, `gio open origa://auth/callback?code=…` starts the app with an
/// empty argv — no `deep-link://new-url` event reaches the running instance
/// and the login page never closes. The plugin's runtime handler carries `%u`
/// and forwards the URL; once that handler is retired on Linux, a repo-owned
/// static registration must carry `%u` instead.
#[test]
fn linux_scheme_default_forwards_the_url() {
    let lib_rs = include_str!("../src/lib.rs");

    // The runtime path still forwards URLs (the plugin writes `%u` into its
    // handler). The Windows-only guard above is the authority on whether
    // that path may exist — if Linux was re-enabled there, this pin defers to
    // it and the two guards decide together.
    let call = lib_rs
        .find("app.deep_link().register_all()")
        .expect("register_all() call missing from lib.rs — the deep-link gate moved?");
    let cfg_start = lib_rs[..call]
        .rfind("#[cfg(")
        .expect("cfg attribute above register_all() missing");
    if lib_rs[cfg_start..call].contains("linux") {
        return;
    }

    // Static path: one of the repo-owned packaging sources must declare the
    // scheme together with a URL field code, otherwise GIO silently eats the
    // OAuth callback.
    let mut sources = Vec::new();
    for dir in ["tauri", "packaging", ".github", "scripts"] {
        collect_text_files(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .expect("repo root exists above the tauri manifest dir")
                .join(dir),
            &mut sources,
        );
    }
    let declares_scheme = |text: &str| text.contains("x-scheme-handler/origa");
    assert!(
        sources
            .iter()
            .any(|text| declares_scheme(text) && text.contains("%u")),
        "Linux has no runtime deep-link registration, so the static scheme \
         default must forward the URL: a repo-owned .desktop template or a \
         packaging patch has to declare `x-scheme-handler/origa` together \
         with `%u` in Exec. Plain `Exec=origa-app` drops the OAuth callback \
         URL in GIO (reproduced with `gio open` — the app gets no argv) and \
         leaves the user on the login page (uwuwu#1098)."
    );
}

/// The packaged `Origa.desktop` must be generated from the repo-owned template
/// that forwards the URL: both Linux bundle channels (deb, rpm) have to point
/// `desktopTemplate` at the template, and the template itself has to carry
/// `%u` in Exec plus the `origa://` scheme declaration. A dead reference (a
/// template nobody renders) would regress the tester's live finding silently.
#[test]
fn packaged_desktop_entry_is_generated_from_the_url_forwarding_template() {
    let conf: serde_json::Value = serde_json::from_str(include_str!("../tauri.conf.json"))
        .expect("tauri.conf.json is valid JSON");
    let channel_template = |channel: &str| {
        conf.pointer(&format!("/bundle/linux/{channel}/desktopTemplate"))
            .and_then(|v| v.as_str())
            .unwrap_or_default()
    };

    assert_eq!(
        channel_template("deb"),
        "./desktop-entry.desktop.tpl",
        "the deb bundle must render the repo-owned desktop template — the \
         generated default (Exec without %u) drops the OAuth callback URL"
    );
    assert_eq!(
        channel_template("rpm"),
        "./desktop-entry.desktop.tpl",
        "the rpm bundle must render the repo-owned desktop template — the \
         generated default (Exec without %u) drops the OAuth callback URL"
    );

    let template = include_str!("../desktop-entry.desktop.tpl");
    assert!(
        template.contains("Exec={{exec}} %u"),
        "the desktop template must forward the URL (%u) — plain Exec makes \
         GIO silently drop the deep-link URI"
    );
    assert!(
        template.contains("x-scheme-handler/origa"),
        "the desktop template must declare the origa:// scheme"
    );
}

fn collect_text_files(dir: std::path::PathBuf, out: &mut Vec<String>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if path.file_name().is_some_and(|name| name == "target") {
                continue;
            }
            collect_text_files(path, out);
        } else if path
            .file_name()
            .is_some_and(|name| name == "deep_link_gate.rs")
        {
            // The pin's own source mentions both markers — don't self-match.
            continue;
        } else if let Ok(text) = std::fs::read_to_string(path) {
            out.push(text);
        }
    }
}
