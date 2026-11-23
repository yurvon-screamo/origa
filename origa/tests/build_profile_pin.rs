//! Build-profile pin: dev dependencies must compile with `opt-level = 2`.
//!
//! The `[profile.dev.package."*"]` section of the root `Cargo.toml` is the
//! single carrier of the dev-runtime speed contract (task #1467, owner
//! mandate 21.11.2026): non-workspace dependencies are optimized, while
//! workspace members stay unoptimized (`"*"` never matches members) so the
//! incremental edit-rebuild loop of our own crates is unaffected.
//!
//! Profile lines are a classic silent merge-conflict casualty: dropping the
//! key does not break a single test — every run just gets slower again. This
//! pin fails loudly instead.
//!
//! Values are compared strictly: exact key, value up to an inline `#`
//! comment, compared verbatim. A loose `contains('2')` match stays green on
//! `opt-level = 3 # 2`-style rewrites (testing review 22.11).
//!
//! Run: `cargo test -p origa --test build_profile_pin`.

use std::fs;
use std::path::PathBuf;

fn root_cargo_toml() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crate dir must have a parent (workspace root)")
        .join("Cargo.toml")
}

/// Returns the body of a `[section]` as trimmed non-comment lines.
fn section(source: &str, header: &str) -> Vec<String> {
    let mut lines = source.lines();
    for line in lines.by_ref() {
        if line.trim() == header {
            break;
        }
    }
    let mut body = Vec::new();
    for line in lines {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            break;
        }
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        body.push(trimmed.to_string());
    }
    body
}

/// Drops an inline `# comment` from a TOML value. The values this pin reads
/// (integers, booleans) never contain `#`, so a plain split is exact.
fn without_inline_comment(value: &str) -> &str {
    match value.split_once('#') {
        Some((before, _)) => before,
        None => value,
    }
}

/// Value of `key` inside a section body: exact key match (`key = value`),
/// inline comment stripped, whitespace trimmed. `None` when the key is absent.
fn setting<'a>(section: &'a [String], key: &str) -> Option<&'a str> {
    section
        .iter()
        .filter_map(|line| line.split_once('='))
        .find(|(k, _)| k.trim() == key)
        .map(|(_, v)| without_inline_comment(v).trim())
}

#[test]
fn setting_strips_inline_comment_from_value() {
    // Arrange: a rewritten profile line whose comment still carries the old
    // value — the false-green class the loose `contains('2')` check missed.
    let lines = vec!["opt-level = 3 # was 2".to_string()];

    // Act + Assert
    assert_eq!(setting(&lines, "opt-level"), Some("3"));
}

#[test]
fn setting_matches_exact_key_only() {
    // Arrange: a prefixed key must not satisfy a lookup for the exact one.
    let lines = vec![
        "opt-level-suffix = 2".to_string(),
        "opt-level = 0".to_string(),
    ];

    // Act + Assert
    assert_eq!(setting(&lines, "opt-level"), Some("0"));
    assert_eq!(setting(&lines, "opt-level-suffix"), Some("2"));
}

#[test]
fn setting_missing_key_returns_none() {
    // Arrange
    let lines = vec!["debug = false".to_string()];

    // Act + Assert
    assert_eq!(setting(&lines, "opt-level"), None);
}

#[test]
fn setting_tolerates_whitespace_around_value() {
    // Arrange: both spellings are TOML-legal.
    let lines = vec!["opt-level=2".to_string(), "debug =   false  ".to_string()];

    // Act + Assert
    assert_eq!(setting(&lines, "opt-level"), Some("2"));
    assert_eq!(setting(&lines, "debug"), Some("false"));
}

#[test]
fn dev_deps_are_optimized() {
    let source = fs::read_to_string(root_cargo_toml())
        .expect("root Cargo.toml must be readable from any workspace member");
    let section = section(&source, r#"[profile.dev.package."*"]"#);
    assert!(
        !section.is_empty(),
        r#"section [profile.dev.package."*"] is missing from the root Cargo.toml"#
    );

    let opt_level = setting(&section, "opt-level");
    assert!(
        opt_level == Some("2"),
        "dev dependencies lost their optimization: `opt-level = 2` must be set \
         verbatim in [profile.dev.package.\"*\"] (found {opt_level:?}) — dev \
         test/E2E runs regress to opt-level 0 runtime (task #1467)"
    );
    let debug_info = setting(&section, "debug");
    assert!(
        debug_info == Some("false"),
        "dep debuginfo policy (`debug = false`) must be set verbatim in \
         [profile.dev.package.\"*\"] (found {debug_info:?})"
    );
}

#[test]
fn workspace_members_stay_unoptimized() {
    // The parent `[profile.dev]` must not carry an `opt-level` key: workspace
    // members are intentionally left on opt-level 0 + incremental to keep the
    // edit-rebuild loop fast. Speeding up own-crate runtime belongs in the
    // `[profile.dev.package."*"]` override, never in the shared profile.
    let source = fs::read_to_string(root_cargo_toml())
        .expect("root Cargo.toml must be readable from any workspace member");
    let section = section(&source, "[profile.dev]");
    assert!(
        !section.is_empty(),
        "section [profile.dev] is missing from the root Cargo.toml"
    );
    assert!(
        section.iter().all(|l| !l.starts_with("opt-level")),
        "[profile.dev] gained an `opt-level` key — workspace members must stay \
         opt-level 0 + incremental (the \"*\" override covers dependencies only)"
    );
}
