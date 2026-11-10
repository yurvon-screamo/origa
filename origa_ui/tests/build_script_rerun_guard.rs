//! Cache guard for the `origa_ui` build script (task 20261109020526-ej6jeho).
//!
//! `cargo:rerun-if-changed` on a path that does not exist makes cargo re-dirty
//! the build script on EVERY invocation (observed on cargo 1.98: `Dirty
//! origa_ui: the file ../cdn/fonts is missing`). The whole crate then
//! recompiles on every build in any checkout without the gitignored `cdn/`
//! assets — fresh worktrees, CI lint/test checkouts. The fix guards both font
//! watch prints behind a `is_dir` check; this suite pins the emitted build
//! script output against the resolved fonts source (the mirror of
//! `resolve_fonts_dir`: `ORIGA_CDN_CACHE_DIR` corpus when it carries `fonts/`,
//! else the repo-local `cdn/fonts` — task 20261110041750-4bs23o3) so a
//! regression (unconditional print, or a watch pointing at the wrong source)
//! fails instead of silently costing ~30 s per build.
//!
//! Reads the freshest cached build script output under `target/`: `cargo test`
//! compiles `origa_ui` before running this suite, so that file always
//! reflects the current `build.rs` source and the current env (`cargo`
//! re-runs the script when `ORIGA_CDN_CACHE_DIR` changes — the
//! `rerun-if-env-changed` contract pinned by `env_watches_declared_for_read_env_vars`).
//! A host suite without any fonts source pins the "missing dir -> no watch"
//! branch; a suite with the env corpus (or the downloaded CDN artifact)
//! pins the "source exists -> watch emitted" branch.

#![cfg(not(target_arch = "wasm32"))] // std::fs does not exist on wasm32 (wasm-pack compiles this suite too)

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

fn cdn_fonts_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("cdn")
        .join("fonts")
}

/// The fonts source the build script resolves right now — the test-side mirror
/// of `resolve_fonts_dir` in build.rs: a non-empty `ORIGA_CDN_CACHE_DIR`
/// carrying `fonts/` wins, else the repo-local `cdn/fonts`. `None` — no
/// source anywhere, the watch must be absent from the output.
fn resolved_fonts_dir() -> Option<PathBuf> {
    if let Ok(cache_dir) = std::env::var("ORIGA_CDN_CACHE_DIR") {
        if !cache_dir.is_empty() {
            let from_env = Path::new(&cache_dir).join("fonts");
            if from_env.is_dir() {
                return Some(from_env);
            }
        }
    }
    let repo_local = cdn_fonts_dir();
    repo_local.is_dir().then_some(repo_local)
}

/// Freshest cached output of the `origa_ui` build script under the host
/// profiles of the workspace `target/`. Layout:
/// `target/{profile}/build/origa_ui-<hash>/output`.
///
/// Host profiles only (`debug`, `release`): the triple dirs
/// (`target/wasm32-unknown-unknown/...`) are trunk/cross builds with their
/// own env history — a capture run hours ago must not outvote the current
/// host run (task 20261110041750-4bs23o3 review). Within the host profile
/// the invariant holds: every env switch dirties the script (`cargo` reports
/// `the env variable ORIGA_CDN_CACHE_DIR changed`) and rewrites the current
/// fingerprint's output, so the freshest host output always reflects the
/// env `cargo test` just ran with.
fn freshest_script_output() -> Option<PathBuf> {
    let target = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("target");
    let mut best: Option<(SystemTime, PathBuf)> = None;
    for profile in ["debug", "release"] {
        for candidate in build_dirs_under(&target.join(profile)) {
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
    }
    best.map(|(_, path)| path)
}

/// All `build/origa_ui-<hash>` script dirs one level below the given
/// profile dir.
fn build_dirs_under(profile_dir: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let build = profile_dir.join("build");
    let Ok(scripts) = fs::read_dir(&build) else {
        return found;
    };
    for script in scripts.flatten() {
        let name = script.file_name();
        if name.to_string_lossy().starts_with("origa_ui-") && script.path().is_dir() {
            found.push(script.path());
        }
    }
    found
}

fn freshest_output_content() -> Option<String> {
    let output = freshest_script_output()?;
    fs::read_to_string(output).ok()
}

/// The directive prefix of a `cargo:rerun-if-changed` line, as a path.
fn watch_path(line: &str) -> Option<&Path> {
    line.strip_prefix("cargo:rerun-if-changed=").map(Path::new)
}

/// The font watch must point exactly at the resolved fonts source: emitted
/// with that path when the source exists (font changes invalidate the cache),
/// absent when it does not (a watch on a missing path re-dirties the build
/// script on every cargo run). Paths compare by components, not by display
/// strings: `display()` prints platform separators (`\fonts` on a native
/// Windows host), and a `cdn/fonts` substring once made this test reject the
/// healthy env-source workflow the capture stack documents (task
/// 20261110041750-4bs23o3).
#[test]
fn font_watch_matches_fonts_source_state() {
    let Some(content) = freshest_output_content() else {
        eprintln!(
            "skip: no origa_ui build script output under target/ — cargo test \
             compiles origa_ui first, so this only happens outside a cargo test run"
        );
        return;
    };
    // The font watch lines point at a path named `fonts` (registry + staging
    // print the same resolved dir); every other watch targets a file or a
    // different directory.
    let watched = content
        .lines()
        .map(str::trim)
        .filter_map(watch_path)
        .find(|path| path.file_name() == Some(OsStr::new("fonts")));
    match resolved_fonts_dir() {
        Some(dir) => assert!(
            watched == Some(dir.as_path()),
            "build script output watches {watched:?} but the resolved fonts source \
             is `{}` — the watch must follow ORIGA_CDN_CACHE_DIR with the \
             repo-local cdn/fonts fallback (resolve_fonts_dir in origa_ui/build.rs), \
             otherwise font updates may not invalidate the origa_ui cache",
            dir.display()
        ),
        None => assert!(
            watched.is_none(),
            "build script output watches the missing fonts source ({watched:?}) — \
             cargo re-dirties the build script on every invocation and origa_ui \
             recompiles on every build in checkouts without the fonts assets; the \
             watch print must stay guarded by is_dir (task 20261109020526-ej6jeho)"
        ),
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
        "ORIGA_CDN_CACHE_DIR",
        "ORIGA_CDN_REGION",
        "TRAILBASE_URL",
        "SENTRY_DSN",
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
