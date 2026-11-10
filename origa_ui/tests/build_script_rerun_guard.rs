//! Cache guard for the `origa_ui` build script (task 20261109020526-ej6jeho).
//!
//! `cargo:rerun-if-changed` on a path that does not exist makes cargo re-dirty
//! the build script on EVERY invocation (observed on cargo 1.98: `Dirty
//! origa_ui: the file ../cdn/fonts is missing`). The whole crate then
//! recompiles on every build in any checkout without the gitignored `cdn/`
//! assets — fresh worktrees, CI lint/test checkouts. The fix guards both font
//! watch prints behind a `is_dir` check; this suite pins the emitted build
//! script output against the actual state of `cdn/fonts` so a regression
//! (unconditional print) fails instead of silently costing ~30 s per build.
//!
//! Reads the freshest cached build script output under `target/`: `cargo test`
//! compiles `origa_ui` before running this suite, so that file always
//! reflects the current `build.rs` source. Host suites (no `cdn/` locally)
//! pin the "missing dir -> no watch" branch; CI suites run with the CDN
//! artifact downloaded and pin the "dir exists -> watch emitted" branch.

#![cfg(not(target_arch = "wasm32"))] // std::fs does not exist on wasm32 (wasm-pack compiles this suite too)

use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

fn cdn_fonts_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("cdn")
        .join("fonts")
}

/// Freshest cached output of the `origa_ui` build script anywhere under the
/// workspace `target/`. Layout: `target/{triple,profile}/build/origa_ui-<hash>/output`.
fn freshest_script_output() -> Option<PathBuf> {
    let target = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("target");
    let mut best: Option<(SystemTime, PathBuf)> = None;
    for candidate in build_dirs_under(&target) {
        let out = candidate.join("output");
        let Ok(meta) = fs::metadata(&out) else {
            continue;
        };
        let Ok(modified) = meta.modified() else {
            continue;
        };
        let is_fresher = match &best {
            Some((t, _)) => modified > *t,
            None => true,
        };
        if is_fresher {
            best = Some((modified, out));
        }
    }
    best.map(|(_, path)| path)
}

/// All `build/origa_ui-<hash>` script dirs one and two levels below `target/`
/// (host profile directly, target triples one level deeper).
fn build_dirs_under(target: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let Ok(entries) = fs::read_dir(target) else {
        return found;
    };
    for level1 in entries.flatten() {
        let path = level1.path();
        if !path.is_dir() {
            continue;
        }
        for level2 in fs::read_dir(&path).into_iter().flatten().flatten() {
            let build = level2.path().join("build");
            let Ok(scripts) = fs::read_dir(&build) else {
                continue;
            };
            for script in scripts.flatten() {
                let name = script.file_name();
                if name.to_string_lossy().starts_with("origa_ui-") && script.path().is_dir() {
                    found.push(script.path());
                }
            }
        }
    }
    found
}

fn freshest_output_content() -> Option<String> {
    let output = freshest_script_output()?;
    fs::read_to_string(output).ok()
}

/// The font watch must match the actual state of `cdn/fonts`: emitted when the
/// dir exists (font changes invalidate the cache), absent when it does not
/// (a watch on a missing path re-dirties the build script on every cargo run).
#[test]
fn font_watch_matches_cdn_fonts_state() {
    let Some(content) = freshest_output_content() else {
        eprintln!(
            "skip: no origa_ui build script output under target/ — cargo test \
             compiles origa_ui first, so this only happens outside a cargo test run"
        );
        return;
    };
    let watch_emitted = content
        .lines()
        .any(|line| line.contains("rerun-if-changed") && line.contains("cdn/fonts"));
    if cdn_fonts_dir().is_dir() {
        assert!(
            watch_emitted,
            "cdn/fonts exists but the build script output has no rerun-if-changed \
             watch for it — font updates no longer invalidate the origa_ui cache; \
             see generate_font_registry/stage_fonts_for_bundling in origa_ui/build.rs"
        );
    } else {
        assert!(
            !watch_emitted,
            "build script output watches the missing cdn/fonts — cargo re-dirties \
             the build script on every invocation and origa_ui recompiles on every \
             build in checkouts without cdn assets; the watch print must stay \
             guarded by is_dir (task 20261109020526-ej6jeho)"
        );
    }
}

/// Env watches are the invalidation contract read by build.rs for every
/// `option_env!`/`env!` value it emits (ADR-024). Losing one silently bakes a
/// stale URL into the wasm bundle after an env change — the failure mode this
/// task originally investigated (turned out the directives were all in place;
/// this pins that they stay that way).
#[test]
fn env_watches_declared_for_read_env_vars() {
    let Some(content) = freshest_output_content() else {
        eprintln!("skip: no origa_ui build script output under target/");
        return;
    };
    for var in [
        "ORIGA_CDN_BASE_URL",
        "ORIGA_CDN_REGION",
        "ORIGA_COMMIT",
        "ORIGA_BUILD_DATE",
        "ORIGA_PUBLIC_BASE_URL",
        "ORIGA_VERSION",
        "SENTRY_DSN",
        "SENTRY_ENVIRONMENT",
        "TRAILBASE_URL",
        "UMAMI_DISABLED",
        "UMAMI_WEBSITE_ID",
    ] {
        let expected = format!("cargo:rerun-if-env-changed={var}");
        assert!(
            content.lines().any(|line| line.trim() == expected),
            "build script output is missing `{expected}` — env changes no longer \
             invalidate origa_ui and a stale value stays baked into the wasm bundle; \
             see the rerun-if-env-changed block in origa_ui/build.rs"
        );
    }
}
