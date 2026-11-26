//! Drift guards for the formatter scope established in uwuwu#966 / PR #977.
//!
//! `tauri/gen/**` is a hand-edited tree (ADR-033 §5, `_build-tauri-apple.yml`).
//! Prettier must not be its last writer: the baseline reformat of PR #904 added
//! a trailing newline to the hand-maintained XcodeGen `project.yml`, and a
//! future formatter run would add it again. CI enforces the scope through the
//! prettier plugin's own `.prettierignore` (qlty triage does not apply to
//! formatter runs), so the contract is pinned here with `include_str!` /
//! `include_bytes!`: removing the ignore entry or reformatting the file makes
//! these tests fail even when the file is absent from a PR's changed-file delta.

/// Active (non-blank, non-comment) lines of the repository-root
/// `.prettierignore` — the file prettier itself honours in the CI gate.
fn active_patterns() -> Vec<&'static str> {
    include_str!("../../.prettierignore")
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect()
}

/// Minimal glob semantics for the patterns this repo uses: a trailing `/**`
/// matches the directory prefix and everything below it. Deliberately narrow —
/// these guards cover the tauri/gen scope, not a general prettier reimplementation.
fn is_ignored(path: &str, patterns: &[&str]) -> bool {
    patterns.iter().any(|pattern| {
        let prefix = pattern.strip_suffix("/**").unwrap_or(pattern);
        path == prefix || path.starts_with(&format!("{prefix}/"))
    })
}

/// Representative files of the hand-edited tree: the XcodeGen spec and the two
/// JSON resources that prettier can parse. All three must stay out of its scope.
#[test]
fn prettier_ignores_the_hand_maintained_tauri_gen_tree() {
    let patterns = active_patterns();

    for path in [
        "tauri/gen/apple/project.yml",
        "tauri/gen/apple/Assets.xcassets/Contents.json",
        "tauri/gen/apple/Assets.xcassets/AppIcon.appiconset/Contents.json",
    ] {
        assert!(
            is_ignored(path, &patterns),
            "{path} must be out of prettier scope (`.prettierignore` active patterns: {patterns:?})"
        );
    }
}

/// The exclude is for the scaffold directory only. A future blanket `tauri/**`
/// would silently drop the formatter gate from the app config and capabilities,
/// which are actively maintained and must stay checked.
#[test]
fn prettier_scope_does_not_swallow_other_tauri_configs() {
    let patterns = active_patterns();

    for path in ["tauri/tauri.conf.json", "tauri/capabilities/default.json"] {
        assert!(
            !is_ignored(path, &patterns),
            "{path} must stay in prettier scope (`.prettierignore` active patterns: {patterns:?})"
        );
    }
}

/// Byte pin: `project.yml` is a hand-maintained XcodeGen spec whose last manual
/// state ends without a newline. The baseline reformat of PR #904 added one;
/// that format-only tail must not come back through a formatter run.
#[test]
fn project_yml_keeps_the_hand_maintained_no_trailing_newline() {
    let bytes = include_bytes!("../gen/apple/project.yml");

    assert!(
        !bytes.ends_with(b"\n"),
        "tauri/gen/apple/project.yml must end without a trailing newline \
         (uwuwu#966 returned it to the last manual state; prettier added one in PR #904)"
    );
    assert!(
        bytes.ends_with(b"libapp.a"),
        "the pinned tail is the arm64 libapp.a outputFiles entry"
    );
}
