//! Grammar corpus pin gate — deploy-window stability for corpus tests.
//!
//! CI downloads the grammar corpus from prod (the `cdn-download` job) before
//! the fixtures PR lands (deploy-before-merge doctrine), so corpus tests on
//! arbitrary branches would compare a fresh corpus against stale in-code
//! pins and fail on diffs that never touched the grammar. The pin gate
//! skips those tests while the corpus deploy window is open: the downloaded
//! corpus's SHA256 must match the pin committed in
//! `tests/fixtures/cdn_grammar_v3.sha256` — the fixtures PR bumps the pin
//! alongside the content pins it re-derives from the new corpus.
//!
//! An absent corpus keeps the existing graceful-skip convention (fresh
//! clones, `cdn/` gitignored); only a hash mismatch skips with the drift
//! marker, which carries seen/expected hashes and the fix path.

use std::path::{Path, PathBuf};

use origa::dictionary::cdn_blob::sha256_bytes;

/// Corpus path relative to the workspace-root `cdn/` checkout.
pub const GRAMMAR_CORPUS_REL_PATH: &str = "grammar/grammar_v3.json";

/// Compiled-in pin (`<hex>  grammar/grammar_v3.json`, sha256sum format) —
/// part of the commit, like the content pins it travels with.
const PIN_FILE: &str = include_str!("../fixtures/cdn_grammar_v3.sha256");

/// Workspace-root `cdn/` directory (parent of the `origa` crate manifest).
pub fn cdn_dir() -> PathBuf {
    let manifest_dir =
        std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is set by cargo");
    PathBuf::from(manifest_dir)
        .parent()
        .expect("workspace root is parent of the origa crate manifest")
        .join("cdn")
}

/// The pinned corpus SHA256 (first whitespace-separated token of the pin file).
pub fn pinned_corpus_sha256() -> &'static str {
    PIN_FILE
        .split_whitespace()
        .next()
        .expect("pin file must carry a hash token")
}

/// Lowercase hex encoding of the SHA256 digest (sha256sum-compatible).
pub fn sha256_hex(bytes: &[u8]) -> String {
    sha256_bytes(bytes)
        .iter()
        .fold(String::with_capacity(64), |mut out, byte| {
            use std::fmt::Write as _;
            let _ = write!(out, "{byte:02x}");
            out
        })
}

/// SHA256 of the downloaded corpus file; `None` when absent — absence is a
/// caller concern (graceful-skip convention), not drift.
pub fn downloaded_corpus_sha256(cdn_root: &Path) -> Option<String> {
    std::fs::read(cdn_root.join(GRAMMAR_CORPUS_REL_PATH))
        .ok()
        .map(|bytes| sha256_hex(&bytes))
}

/// True when the downloaded corpus is present but NOT the pinned version —
/// the corpus deploy window is open. Prints the drift marker (seen /
/// expected / fix path) on every drifted call so each skipped test names
/// its own skip; callers then skip their corpus tests silently.
pub fn corpus_pin_drifted(test_label: &str) -> bool {
    corpus_pin_drifted_at(&cdn_dir(), test_label)
}

/// Path-injected variant of [`corpus_pin_drifted`] — the testable core.
pub fn corpus_pin_drifted_at(cdn_root: &Path, test_label: &str) -> bool {
    let Some(seen) = downloaded_corpus_sha256(cdn_root) else {
        return false;
    };
    let expected = pinned_corpus_sha256();
    if seen == expected {
        return false;
    }
    eprintln!(
        "[skip] {test_label}: corpus drift — cdn/{GRAMMAR_CORPUS_REL_PATH} sha256 \
         {seen} != pinned {expected}; corpus deploy window open — update \
         origa/tests/fixtures/cdn_grammar_v3.sha256 in the fixtures PR"
    );
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The pin must be a sha256sum-style line: a 64-char lowercase hex
    /// hash. A malformed pin would make the gate treat every deploy as a
    /// window (or every window as a match) — fail loudly at commit time.
    #[test]
    fn pin_file_carries_a_64_char_lowercase_hex_sha256() {
        let pinned = pinned_corpus_sha256();

        assert_eq!(
            pinned.len(),
            64,
            "pinned sha256 must be 64 hex chars: {pinned}"
        );
        assert!(
            pinned
                .chars()
                .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()),
            "pinned sha256 must be lowercase hex: {pinned}"
        );
    }

    /// The pin references the corpus path the gate hashes — protects
    /// against retargeting the pin file at a different corpus silently.
    #[test]
    fn pin_file_references_the_grammar_v3_corpus() {
        assert!(
            PIN_FILE.trim_end().ends_with(GRAMMAR_CORPUS_REL_PATH),
            "pin file must reference cdn/{GRAMMAR_CORPUS_REL_PATH}, got: {PIN_FILE:?}"
        );
    }

    /// Known SHA256 test vector — guards the hex encoder against
    /// endianness / formatting regressions (sha256sum of "abc").
    #[test]
    fn sha256_hex_matches_the_standard_test_vector() {
        let digest = sha256_hex(b"abc");

        assert_eq!(
            digest,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    /// An absent corpus is not drift — the graceful-skip convention (fresh
    /// clones without the gitignored `cdn/`) must keep working.
    #[test]
    fn absent_corpus_yields_no_hash_and_no_drift() {
        let missing = Path::new("/nonexistent/corpus-pin-probe");

        assert!(downloaded_corpus_sha256(missing).is_none());
        assert!(!corpus_pin_drifted_at(missing, "probe"));
    }

    /// A corpus whose hash differs from the pin is drift — the deploy
    /// window is open and callers must skip.
    #[test]
    fn mismatched_corpus_is_drift() {
        let probe_dir =
            std::env::temp_dir().join(format!("origa-pin-probe-{}", std::process::id()));
        std::fs::create_dir_all(&probe_dir).expect("probe dir must be creatable");
        let corpus_dir = probe_dir.join("grammar");
        std::fs::create_dir_all(&corpus_dir).expect("probe grammar dir must be creatable");
        let corpus_path = corpus_dir.join(
            GRAMMAR_CORPUS_REL_PATH
                .split('/')
                .next_back()
                .expect("relative path has a file name"),
        );
        std::fs::write(&corpus_path, b"probe bytes").expect("probe corpus must be writable");
        assert_ne!(
            downloaded_corpus_sha256(&probe_dir).as_deref(),
            Some(pinned_corpus_sha256()),
            "probe bytes must not accidentally match the pin"
        );

        let drifted = corpus_pin_drifted_at(&probe_dir, "probe");

        let _ = std::fs::remove_dir_all(&probe_dir);
        assert!(drifted, "a non-pinned corpus must report drift");
    }
}
