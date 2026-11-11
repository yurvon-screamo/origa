//! Cache guard for the `origa_ui` build script (task 20261109020526-ej6jeho).
//!
//! `cargo:rerun-if-changed` on a path that does not exist makes cargo re-dirty
//! the build script on EVERY invocation (observed on cargo 1.98: `Dirty
//! origa_ui: the file ../cdn/fonts is missing`). The whole crate then
//! recompiles on every build in any checkout without the seeded corpus —
//! fresh worktrees, CI lint/test checkouts. The fix guards the font and
//! well_known_set watch prints behind `is_dir` checks (the well_known_set
//! corpus watches landed with task 20261109234301-dd6s9tv); this suite pins
//! the emitted build script output against the resolved corpus state (the
//! corpus-root mirror of build.rs `cdn_dir`: `$ORIGA_CDN_DIR` override, else
//! `~/origa-cdn` — task 20261110041750-4bs23o3) so a regression
//! (unconditional print, or a watch pointing at the wrong source) fails
//! instead of silently costing ~30 s per build.
//!
//! Reads the freshest cached build script output under the host profiles of
//! `target/`: `cargo test` compiles `origa_ui` before running this suite, so
//! that file always reflects the current `build.rs` source and the current
//! env (`cargo` re-runs the script when `ORIGA_CDN_DIR` changes — the
//! `rerun-if-env-changed` contract pinned by `env_watches_declared_for_read_env_vars`).
//! Host profiles only (`debug`, `release`): triple dirs carry trunk/cross
//! builds with their own env history — a stale wasm output must not outvote
//! the current host run. A host suite without any fonts source pins the
//! "missing dir -> no watch" branch; a suite with the seeded corpus pins
//! the "source exists -> watch emitted" branch. The pins compare the output
//! of the LAST build script run against the current corpus state: adding
//! corpus files without a build in between leaves a stale output (appearance
//! does not trigger a rerun by itself) — rebuild before judging a red pin in
//! that situation.

#![cfg(not(target_arch = "wasm32"))] // std::fs does not exist on wasm32 (wasm-pack compiles this suite too)

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Corpus root mirror of build.rs `cdn_dir` (the build script cannot import
/// the crate): `$ORIGA_CDN_DIR` override first (empty counts as unset), then
/// `~/origa-cdn` (HOME, then USERPROFILE).
fn cdn_corpus_root() -> PathBuf {
    if let Some(dir) = std::env::var_os("ORIGA_CDN_DIR").filter(|value| !value.is_empty()) {
        return PathBuf::from(dir);
    }
    let home = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("origa-cdn"));
    home.join("origa-cdn")
}

fn cdn_fonts_dir() -> PathBuf {
    cdn_corpus_root().join("fonts")
}

fn cdn_well_known_set_dir() -> PathBuf {
    cdn_corpus_root().join("well_known_set")
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
/// `the env variable ORIGA_CDN_DIR changed`) and rewrites the current
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
/// healthy corpus workflow — a custom `ORIGA_CDN_DIR` (`/mnt/corpus/fonts`)
/// does not even contain that substring (task 20261110041750-4bs23o3).
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
    let fonts_dir = cdn_fonts_dir();
    if fonts_dir.is_dir() {
        assert!(
            watched == Some(fonts_dir.as_path()),
            "build script output watches {watched:?} but the resolved fonts source \
             is `{}` — the watch must follow ORIGA_CDN_DIR with the ~/origa-cdn \
             fallback (cdn_dir in origa_ui/build.rs), otherwise font updates may \
             not invalidate the origa_ui cache",
            fonts_dir.display()
        );
    } else {
        assert!(
            watched.is_none(),
            "build script output watches the missing fonts source ({watched:?}) — \
             cargo re-dirties the build script on every invocation and origa_ui \
             recompiles on every build in checkouts without the fonts assets; the \
             watch print must stay guarded by is_dir (task 20261109020526-ej6jeho)"
        );
    }
}

/// The corpus watches must match the actual contents of `cdn/well_known_set`:
/// corpus children present -> watches emitted (corpus updates invalidate the
/// cache); no children (empty or absent dir) -> no watches. A watch on a
/// missing path re-dirties the build script on every cargo run — the same
/// failure class as the font watch above. The exact per-child correspondence
/// is the generator's input contract (see the watch list in build.rs); this
/// pin only asserts the presence agreement, which covers both regressions:
/// an unconditional print (watch for an empty/absent dir) and a dropped
/// watch (children present but nothing invalidates the cache).
#[test]
fn well_known_watch_matches_corpus_state() {
    let Some(content) = freshest_output_content() else {
        eprintln!("skip: no origa_ui build script output under target/");
        return;
    };
    let watch_count = content
        .lines()
        .map(str::trim)
        .filter_map(watch_path)
        // Component check, not a `cdn/well_known_set` substring: a custom
        // ORIGA_CDN_DIR (`/mnt/corpus`) does not contain that substring
        // (same class as the font watch heuristic above).
        .filter(|path| {
            path.components()
                .any(|c| c.as_os_str() == OsStr::new("well_known_set"))
        })
        .count();
    // Generated meta output lives inside the dir but is not a corpus child.
    let child_count = fs::read_dir(cdn_well_known_set_dir())
        .map(|entries| {
            entries
                .flatten()
                .filter(|e| e.file_name().to_string_lossy() != "well_known_sets_meta.json")
                .count()
        })
        .unwrap_or(0);
    if child_count > 0 {
        assert!(
            watch_count > 0,
            "cdn/well_known_set has {} corpus children but the build script \
             output has no rerun-if-changed watch for them — corpus updates no longer \
             invalidate the origa_ui cache; see generate_well_known_meta in \
             origa_ui/build.rs",
            child_count
        );
    } else {
        assert_eq!(
            watch_count, 0,
            "build script output emits {} watch(es) for an absent or empty \
             cdn/well_known_set — cargo re-dirties the build script on every invocation \
             and origa_ui recompiles on every build; the watch prints must stay behind \
             the existence checks (task 20261109234301-dd6s9tv)",
            watch_count
        );
    }
}

/// Env watches are the invalidation contract read by build.rs for every
/// `option_env!`/`env!` value it emits (ADR-024). Losing one silently bakes a
/// stale URL into the wasm bundle after an env change — the failure mode this
/// task originally investigated (turned out the directives were all in place;
/// this pins that they stay that way). `ORIGA_CDN_DIR` joined the list with
/// the font-source wiring (task 20261110041750-4bs23o3): losing its
/// directive silently bakes a stale font registry.
#[test]
fn env_watches_declared_for_read_env_vars() {
    let Some(content) = freshest_output_content() else {
        eprintln!("skip: no origa_ui build script output under target/");
        return;
    };
    for var in [
        "ORIGA_CDN_BASE_URL",
        "ORIGA_CDN_DIR",
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
