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
    origa::corpus::cdn_dir()
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

/// Drift = corpus present and not matching the pin. Absence is not drift —
/// it keeps the existing graceful-skip convention (fresh clones).
pub fn is_corpus_drift(seen: Option<&str>, pinned: &str) -> bool {
    matches!(seen, Some(hash) if hash != pinned)
}

/// True when the downloaded corpus is present but NOT the pinned version —
/// the corpus deploy window is open (CI) or the local `cdn/` copy is stale
/// (both directions print the same marker: without a network probe the
/// direction is not distinguishable). Callers then skip their corpus tests
/// silently.
pub fn corpus_pin_drifted(test_label: &str) -> bool {
    corpus_pin_drifted_at(&cdn_dir(), test_label)
}

/// Path-injected variant of [`corpus_pin_drifted`] — the testable core.
pub fn corpus_pin_drifted_at(cdn_root: &Path, test_label: &str) -> bool {
    let seen = downloaded_corpus_sha256(cdn_root);
    let pinned = pinned_corpus_sha256();
    if !is_corpus_drift(seen.as_deref(), pinned) {
        return false;
    }
    eprintln!(
        "[skip] {test_label}: corpus drift — cdn/{GRAMMAR_CORPUS_REL_PATH} sha256 \
         {} != pinned {pinned}; if the corpus deploy window is open, update \
         origa/tests/fixtures/cdn_grammar_v3.sha256 in the fixtures PR, \
         otherwise refresh your local cdn/ via scripts/deploy_cdn.py",
        seen.expect("drift implies a seen hash")
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

    /// The pure drift core: present-and-different is drift, present-and-
    /// matching is not, absent is not (three outcome branches of the gate).
    #[test]
    fn is_corpus_drift_covers_the_three_outcome_branches() {
        let pinned = pinned_corpus_sha256();

        assert!(!is_corpus_drift(None, pinned), "absence is not drift");
        assert!(
            !is_corpus_drift(Some(pinned), pinned),
            "a matching corpus is not drift"
        );
        assert!(
            is_corpus_drift(Some("deadbeef"), pinned),
            "a mismatching corpus is drift"
        );
    }

    /// The end-to-end path (file on disk → hash → pin comparison) flags a
    /// non-pinned corpus as drift — without the printing wrapper, so the
    /// probe leaves no marker noise in `--nocapture` runs.
    #[test]
    fn mismatched_corpus_file_flags_as_drift() {
        let probe_dir =
            std::env::temp_dir().join(format!("origa-pin-probe-{}", std::process::id()));
        std::fs::create_dir_all(probe_dir.join("grammar")).expect("probe dir must be creatable");
        std::fs::write(probe_dir.join(GRAMMAR_CORPUS_REL_PATH), b"probe bytes")
            .expect("probe corpus must be writable");

        let seen = downloaded_corpus_sha256(&probe_dir);

        let _ = std::fs::remove_dir_all(&probe_dir);
        assert!(is_corpus_drift(seen.as_deref(), pinned_corpus_sha256()));
    }
}
