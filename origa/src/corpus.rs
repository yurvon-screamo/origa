//! Local CDN corpus: location, skip-marker contract.
//!
//! The corpus is machine-local state, deliberately outside the repository:
//! every worktree on a machine shares one seeded copy instead of carrying a
//! private gitignored `cdn/`. The canonical location is `$ORIGA_CDN_DIR`,
//! falling back to `~/origa-cdn` (`HOME`, then `USERPROFILE`). Nothing in
//! the workspace reads repo-root `cdn/` anymore.
//!
//! Skip contract shared by every corpus-reading fixture: when the corpus
//! (or a corpus entry) is absent, the loader does not panic — it emits the
//! skip marker once per test process and leaves its dictionary unloaded,
//! so the calling test can bail out with a skip. A present-but-broken
//! corpus stays a loud loader panic: a real seed bug must not be masked.
//! Seeding recipe: AGENTS.md → «Local corpus».

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

static CORPUS_SKIP_REPORTED: AtomicBool = AtomicBool::new(false);

/// Root of the local CDN corpus: `$ORIGA_CDN_DIR` or `~/origa-cdn`.
pub fn cdn_dir() -> PathBuf {
    resolve_cdn_dir(
        std::env::var_os("ORIGA_CDN_DIR").as_deref(),
        std::env::var_os("HOME").as_deref(),
        std::env::var_os("USERPROFILE").as_deref(),
    )
}

/// Pure resolution core: explicit env override wins (empty string counts as
/// unset), then `HOME`, then `USERPROFILE`; with no home at all the relative
/// `origa-cdn` keeps resolution total — absence of that path then trips the
/// regular skip marker instead of panicking the resolver.
fn resolve_cdn_dir(
    explicit: Option<&OsStr>,
    home: Option<&OsStr>,
    userprofile: Option<&OsStr>,
) -> PathBuf {
    if let Some(dir) = explicit.filter(|value| !value.is_empty()) {
        return PathBuf::from(dir);
    }
    let home = home.or(userprofile);
    match home {
        Some(home) => PathBuf::from(home).join("origa-cdn"),
        None => PathBuf::from("origa-cdn"),
    }
}

/// Emit the corpus-absent marker once per test process. A plain eprintln:
/// libtest captures the test thread's output, so in a default `cargo test`
/// run the marker surfaces only under `--nocapture` or in the report of a
/// failing test; the always-visible signal for an unseeded corpus is the
/// `cargo:warning` from `origa/build.rs`. Public so integration tests
/// (outside the crate, under the same capture) share one marker form.
pub fn print_corpus_skip_marker(what: &str, path: &Path) {
    if CORPUS_SKIP_REPORTED.swap(true, Ordering::SeqCst) {
        return;
    }
    eprintln!(
        "[skip] {what}: local corpus not found at {}\n\
         Corpus-dependent tests are skipped. Seed it once: set $ORIGA_CDN_DIR \
         or seed ~/origa-cdn (AGENTS.md -> \"Local corpus\").",
        path.display()
    );
}

/// Skip probe shared by every corpus-reading fixture: when the corpus entry
/// is absent, emit the marker once and tell the caller to bail out. A
/// present-but-broken corpus falls through to the loader — its panic names
/// the corrupt path (seed bugs are loud, not masked as skips).
#[cfg(test)]
pub(crate) fn corpus_absent(path: &Path, what: &str) -> bool {
    if path.exists() {
        return false;
    }
    print_corpus_skip_marker(what, path);
    true
}

/// Install the five-dictionary fixture family, or early-return the calling
/// test when the local corpus is absent (marker already emitted by the
/// fixture). SudachiDict + vocabulary + kanji + radicals + grammar.
#[cfg(test)]
macro_rules! real_dictionaries_or_skip {
    () => {
        crate::test_support::init_real_dictionaries();
        if !crate::test_support::real_dictionaries_ready() {
            return;
        }
    };
}

/// Install the translation fixture, or early-return the calling test when
/// the local corpus is absent. Deliberately lighter than
/// [`real_dictionaries_or_skip!`]: the translation pipeline reads three
/// dictionaries (SudachiDict + vocabulary + grammar), never kanji/radicals
/// — the guard mirrors the fixture, not the whole family.
#[cfg(test)]
macro_rules! translation_dictionaries_or_skip {
    () => {
        crate::domain::tokenizer::translation::test_env::ensure_dictionaries();
        if !crate::domain::tokenizer::translation::test_env::translation_dictionaries_ready() {
            return;
        }
    };
}

#[cfg(test)]
pub(crate) use {real_dictionaries_or_skip, translation_dictionaries_or_skip};

#[cfg(test)]
mod tests {
    use super::*;

    fn resolve(explicit: Option<&str>, home: Option<&str>, userprofile: Option<&str>) -> PathBuf {
        resolve_cdn_dir(
            explicit.map(OsStr::new),
            home.map(OsStr::new),
            userprofile.map(OsStr::new),
        )
    }

    #[test]
    fn explicit_env_override_wins_over_home() {
        assert_eq!(
            resolve(Some("/data/origa-cdn"), Some("/home/tester"), None),
            PathBuf::from("/data/origa-cdn")
        );
    }

    #[test]
    fn empty_env_override_counts_as_unset() {
        assert_eq!(
            resolve(Some(""), Some("/home/tester"), None),
            PathBuf::from("/home/tester").join("origa-cdn")
        );
    }

    #[test]
    fn home_is_the_default_location() {
        assert_eq!(
            resolve(None, Some("/home/tester"), Some("C:\\Users\\tester")),
            PathBuf::from("/home/tester").join("origa-cdn")
        );
    }

    #[test]
    fn userprofile_backs_home_up_on_windows() {
        assert_eq!(
            resolve(None, None, Some("C:\\Users\\tester")),
            PathBuf::from("C:\\Users\\tester").join("origa-cdn")
        );
    }

    #[test]
    fn missing_home_falls_back_to_a_relative_path() {
        assert_eq!(resolve(None, None, None), PathBuf::from("origa-cdn"));
    }

    #[test]
    fn absent_corpus_entry_probes_as_skip() {
        // Arrange: a corpus entry that does not exist (fresh machine, no seed).
        let missing =
            std::env::temp_dir().join(format!("origa-corpus-absent-probe-{}", std::process::id()));

        // Act + Assert: the probe reports absence so the caller bails out green.
        assert!(corpus_absent(&missing, "unit probe"));
    }

    #[test]
    fn present_corpus_entry_falls_through_to_loader() {
        // Arrange: a corpus entry that exists (seeded machine).
        let dir = tempfile::tempdir().expect("sandbox corpus dir");
        let present = dir.path().join("entry.json");
        std::fs::write(&present, b"{}").expect("sandbox corpus entry");

        // Act + Assert: the loader proceeds — a present-but-broken entry stays
        // a loud loader panic instead of being masked as a skip.
        assert!(!corpus_absent(&present, "unit probe"));
    }
}
