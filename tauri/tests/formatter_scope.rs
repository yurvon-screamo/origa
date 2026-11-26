//! Drift guards for the formatter scope established in uwuwu#966 / PR #977.
//!
//! `tauri/gen/**` is a hand-edited tree (ADR-033 §5, `_build-tauri-apple.yml`).
//! Prettier must not be its last writer: the baseline reformat of PR #904 added
//! a trailing newline to the hand-maintained XcodeGen `project.yml`, and a
//! future formatter run would add it again. CI enforces the scope through the
//! prettier plugin's own `.prettierignore` (qlty triage does not apply to
//! formatter runs), so the contract is pinned here with `include_str!` /
//! `include_bytes!`: removing or neutralizing the ignore entry makes these
//! tests fail even when the file is absent from a PR's changed-file delta.

/// Active (non-blank, non-comment) lines of the repository-root
/// `.prettierignore` — the file prettier itself honours in the CI gate.
///
/// The matcher below models only the syntax used here: a plain path prefix or
/// one trailing `/**`. Trailing spaces and `\r` are trimmed; any other syntax
/// fails loudly instead of silently disagreeing with prettier.
fn active_patterns() -> Vec<&'static str> {
    include_str!("../../.prettierignore")
        .lines()
        .map(str::trim_end)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .inspect(|&line| {
            assert!(
                !line.starts_with('!') && line == line.trim_start(),
                "is_ignored does not model negation or leading whitespace in {line:?}; \
                 extend the matcher or keep the entry in this form"
            );
            let stem = line.strip_suffix("/**").unwrap_or(line);
            assert!(
                !stem
                    .chars()
                    .any(|c| matches!(c, '*' | '?' | '[' | ']' | '{' | '}' | '!' | '\\')),
                "is_ignored does not model {line:?}; \
                 extend the matcher or keep the entry in this form"
            );
        })
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
fn prettierignore_covers_the_hand_maintained_tauri_gen_tree() {
    let patterns = active_patterns();

    for path in [
        "tauri/gen/apple/project.yml",
        "tauri/gen/apple/Assets.xcassets/Contents.json",
        "tauri/gen/apple/Assets.xcassets/AppIcon.appiconset/Contents.json",
    ] {
        assert!(
            is_ignored(path, &patterns),
            "{path} must be covered by `.prettierignore` (active patterns: {patterns:?})"
        );
    }
}

/// The exclude must stay scoped to the scaffold directory. `tauri.conf.json`
/// and `tauri/package.json` are real targets of the formatter gate; a blanket
/// `tauri/**` would silently drop them. `tauri/capabilities/**` is deliberately
/// NOT a formatter target — `.qlty/qlty.toml` excludes it from all scans, and
/// its generator-owned tabs are pinned by `tauri/tests/build_config.rs`.
#[test]
fn prettier_scope_does_not_swallow_other_tauri_configs() {
    let patterns = active_patterns();

    for path in ["tauri/tauri.conf.json", "tauri/package.json"] {
        assert!(
            !is_ignored(path, &patterns),
            "{path} must stay in prettier scope (`.prettierignore` active patterns: {patterns:?})"
        );
    }
}

/// Byte pin: `project.yml` is a hand-maintained XcodeGen spec whose last manual
/// state ends without a newline. The PR #904 baseline reformat added one; a
/// format-only tail must not come back. The content may change by hand, so only
/// non-emptiness and the absence of a trailing newline are pinned.
#[test]
fn project_yml_keeps_the_hand_maintained_no_trailing_newline() {
    let bytes = include_bytes!("../gen/apple/project.yml");

    assert!(
        !bytes.is_empty(),
        "tauri/gen/apple/project.yml must not be empty"
    );
    assert!(
        !bytes.ends_with(b"\n"),
        "tauri/gen/apple/project.yml must not end with a newline \
         (uwuwu#966 returned it to the last manual state; a format-only tail \
         must not be added by a formatter or an editor)"
    );
}
