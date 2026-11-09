//! E2E guard: the deploy-window pin gate must stay WIRED into the corpus
//! binaries' loaders (see ``common/grammar_corpus.rs``).
//!
//! The unit tests inside ``common/grammar_corpus.rs`` cover the gate logic
//! itself, but nothing there proves the gate is still called by the three
//! corpus binaries: delete the ``corpus_pin_drifted`` call from
//! ``ensure_corpus_loaded`` / ``ensure_grammar_loaded`` /
//! ``merge_twins_match_the_shipped_v3_corpus`` and the unit tests stay
//! green while the red-on-foreign-diff deploy-window class returns. These
//! tests spawn each compiled corpus binary with ``ORIGA_CDN_DIR`` — the
//! override the gate's ``cdn_dir()`` resolves first — pointing at a drifted
//! corpus, and assert the binary stays green AND
//! its shipped-corpus tests skipped via the ``[skip] … corpus drift``
//! marker (visible because the harness runs with ``--nocapture``; without
//! it the libtest capture swallows the markers).
//!
//! Cargo only rebuilds the test targets of the suite it runs, so a selective
//! run of just this file may see stale binaries from before recent edits —
//! that would silently validate old code. When the test sources (or crate
//! sources) are newer than the spawned binary, the check skips loudly
//! instead; full-suite runs — including CI — rebuild everything first and
//! never hit this.
//!
//! Run: ``cargo test -p origa --test grammar_pin_gate_e2e``.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;

use rstest::rstest;

fn modified_time(path: &Path) -> SystemTime {
    path.metadata()
        .and_then(|meta| meta.modified())
        .unwrap_or(SystemTime::UNIX_EPOCH)
}

/// Newest mtime under a source directory (recursive); UNIX_EPOCH when the
/// directory is missing — a missing tree can never outrun a built binary.
fn newest_source_time(root: &Path) -> SystemTime {
    let Ok(entries) = std::fs::read_dir(root) else {
        return SystemTime::UNIX_EPOCH;
    };
    let mut newest = SystemTime::UNIX_EPOCH;
    for entry in entries.flatten() {
        let path = entry.path();
        let candidate = if path.is_dir() {
            newest_source_time(&path)
        } else {
            modified_time(&path)
        };
        if candidate > newest {
            newest = candidate;
        }
    }
    newest
}

/// True when the source tree changed after the binary was linked — cargo
/// did not rebuild this target in the current invocation, so spawning it
/// would validate stale code.
fn binary_is_stale(binary: &Path, crate_root: &Path) -> bool {
    let binary_time = modified_time(binary);
    let tests_time = newest_source_time(&crate_root.join("tests"));
    let src_time = newest_source_time(&crate_root.join("src"));
    tests_time > binary_time || src_time > binary_time
}

/// Newest compiled test binary matching ``<target>/debug/deps/<name>-<hash>``
/// (cargo may keep stale hashes around).
fn find_corpus_binary(name: &str) -> Option<PathBuf> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir.parent().expect("crate lives in the workspace");
    let mut target_dir = std::env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| workspace_root.join("target"));
    // A relative CARGO_TARGET_DIR resolves against the workspace root, not cwd.
    if target_dir.is_relative() {
        target_dir = workspace_root.join(target_dir);
    }
    let deps = target_dir.join("debug/deps");
    let mut best: Option<(PathBuf, SystemTime)> = None;
    let entries = std::fs::read_dir(&deps).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        let is_match = path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.starts_with(&format!("{name}-")));
        if !is_match || !path.is_file() {
            continue;
        }
        let modified = modified_time(&path);
        let is_newer = best.as_ref().map_or(true, |(_, time)| modified > *time);
        if is_newer {
            best = Some((path, modified));
        }
    }
    best.map(|(path, _)| path)
}

/// Sandbox with a drifted corpus: `<sandbox>/grammar/grammar_v3.json`
/// exists but is not the pinned version (deploy window open). Returns the
/// sandbox corpus dir — exactly the value the gate's ``cdn_dir()`` reads
/// from the runtime ``ORIGA_CDN_DIR`` override.
fn sandbox_with_drifted_corpus() -> PathBuf {
    let sandbox = std::env::temp_dir().join(format!(
        "origa-pin-gate-e2e-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("clock is after epoch")
            .as_nanos()
    ));
    std::fs::create_dir_all(sandbox.join("grammar")).expect("sandbox must be creatable");
    std::fs::write(
        sandbox.join("grammar/grammar_v3.json"),
        "drifted probe corpus — deliberately not the pinned version",
    )
    .expect("drifted corpus must be writable");
    sandbox
}

#[rstest]
#[case::near_miss("near_miss_coverage")]
#[case::regression("grammar_regression_checks")]
#[case::migration("grammar_migration")]
fn drifted_corpus_skips_with_the_marker_and_stays_green(#[case] binary_name: &str) {
    let Some(binary) = find_corpus_binary(binary_name) else {
        eprintln!(
            "[skip] {binary_name} binary is not built yet — run the corpus test suite once \
             (cargo test -p origa) or build it explicitly, then rerun this file"
        );
        return;
    };
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    if binary_is_stale(&binary, crate_root) {
        eprintln!(
            "[skip] {binary_name} binary predates recent source edits — cargo did not rebuild \
             it in this selective run; run the full corpus suite (cargo test -p origa) so the \
             binary matches the sources, then rerun this file"
        );
        return;
    }

    let sandbox_cdn = sandbox_with_drifted_corpus();
    let output = Command::new(&binary)
        .arg("--nocapture")
        .env("ORIGA_CDN_DIR", &sandbox_cdn)
        .output()
        .expect("corpus binary must spawn");

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let _ = std::fs::remove_dir_all(&sandbox_cdn);

    assert!(
        output.status.success(),
        "{binary_name} must stay green while the deploy window is open \
         (exit {:?}); stderr:\n{stderr}",
        output.status.code()
    );
    assert!(
        stdout.contains("test result: ok"),
        "{binary_name} did not report a green test summary; stdout:\n{stdout}"
    );
    assert!(
        stderr.contains(&format!("[skip] {binary_name}: corpus drift")),
        "{binary_name} executed shipped-corpus tests against a drifted corpus without \
         printing the [skip] corpus drift marker — the pin gate is no longer wired into \
         its loader (red-on-foreign-diff class); stderr:\n{stderr}"
    );
}
