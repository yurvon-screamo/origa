//! Shared corpus/staging helpers for the precompute CLI test binaries.
//!
//! Both suites stage a throwaway corpus outside the seeded CDN: one half
//! probes the skip path without a dictionary, the other rebuilds from the
//! real dictionaries. Keeping the resolution and symlink logic here makes
//! their skip markers and cleanup behave identically.

use std::path::{Path, PathBuf};
use std::process::Output;

use origa::domain::SUDACHIDICT_DIR;

/// Combined stdout/stderr of a spawned builder — assertions quote it on
/// failure instead of hiding the builder's reason behind a bare status.
pub fn output_text(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

/// Corpus roots in the same order the production scripts resolve them:
/// the CI artifact unpack (`<workspace>/cdn`) first, then the machine seed
/// (`$ORIGA_CDN_DIR` / `~/origa-cdn`, ADR-083).
pub fn corpus_roots() -> Vec<PathBuf> {
    let project_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("utils crate lives in the workspace root");
    let mut roots = vec![project_root.join("cdn"), origa::corpus::cdn_dir()];
    roots.dedup();
    roots
}

/// Seeded SudachiDict directory, or `None` when neither root has one.
/// Callers decide whether to skip; this helper never prints a marker, so
/// a caller can name the missing ingredient precisely (`[skip] SudachiDict …`).
pub fn seeded_dictionary_dir() -> Option<PathBuf> {
    corpus_roots()
        .into_iter()
        .map(|root| root.join("dictionaries").join(SUDACHIDICT_DIR))
        .find(|dir| dir.join("metadata.json").is_file())
}

#[cfg(unix)]
pub fn symlink(target: &Path, link: &Path) {
    std::os::unix::fs::symlink(target, link).expect("linking seeded corpus entry into staging");
}

#[cfg(windows)]
pub fn symlink(target: &Path, link: &Path) {
    if target.is_dir() {
        std::os::windows::fs::symlink_dir(target, link)
    } else {
        std::os::windows::fs::symlink_file(target, link)
    }
    .expect("linking seeded corpus entry into staging");
}
