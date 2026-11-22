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

#[test]
fn dev_deps_are_optimized() {
    let source = fs::read_to_string(root_cargo_toml())
        .expect("root Cargo.toml must be readable from any workspace member");
    let section = section(&source, r#"[profile.dev.package."*"]"#);
    assert!(
        !section.is_empty(),
        r#"section [profile.dev.package."*"] is missing from the root Cargo.toml"#
    );

    assert!(
        section
            .iter()
            .any(|l| l.starts_with("opt-level") && l.contains('2')),
        "dev dependencies lost their optimization: `opt-level = 2` is gone \
         from [profile.dev.package.\"*\"] — dev test/E2E runs regress to \
         opt-level 0 runtime (task #1467)"
    );
    assert!(
        section
            .iter()
            .any(|l| l.starts_with("debug") && l.contains("false")),
        "dep debuginfo policy (`debug = false`) is gone from \
         [profile.dev.package.\"*\"]"
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
