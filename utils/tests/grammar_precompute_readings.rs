//! Regression pins for uwuwu#1673: the grammar precompute blob must carry
//! hiragana readings for dictionary-covered kanji words.
//!
//! The blob freshness header binds to `grammar_v3.json`, the SudachiDict
//! directory name and the sha256 of `JmdictFurigana.txt` — not to the
//! annotator code (`utils/README.md` freshness boundary). Rebuilding after
//! PR origa#929 was therefore a manual, forced step: the old blob still
//! looked fresh while carrying katakana readings. These tests pin the
//! readings themselves, at two points:
//!
//! 1. the delivered CDN corpus (the bytes CI downloads from the prod
//!    manifest) — integration with the CDN boundary;
//! 2. a forced rebuild through the real CLI with seeded dictionaries —
//!    the builder/annotator path that produces those bytes.
//!
//! Both follow the repo's corpus skip contract: absent corpus (or a
//! corpus drifted from the version pin) -> `[skip]` marker and early
//! return; present-but-corrupt -> loud failure.

mod common;

use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use common::precompute_support::{corpus_roots, output_text, seeded_dictionary_dir, symlink};
use origa::dictionary::cdn_blob::{SCHEMA_VERSION, sha256_bytes};
use origa::dictionary::precompute_blob::access_precompute_deflated;
use origa::domain::SUDACHIDICT_DIR;

const GRAMMAR_BLOB: &str = "grammar/grammar_precompute_v3.rkyv";
const FURIGANA_SOURCE: &str = "dictionaries/JmdictFurigana.txt";
const GRAMMAR_SOURCE: &str = "grammar/grammar_v3.json";
/// Same version pin the `origa` corpus tests use (AGENTS.md, «Корпус-тесты»).
const GRAMMAR_PIN: &str = include_str!("../../origa/tests/fixtures/cdn_grammar_v3.sha256");
const SAMPLE_RULES: &str = r#"{"rules":[
{"title":"平和"},{"title":"努力"},{"title":"明日"},
{"title":"会社員"},{"title":"図書館"},{"title":"銀行"},{"title":"月曜日"}
]}"#;

/// (kanji surface, reading produced by the current annotator). Only
/// dictionary-covered words are pinned: kana-only tokens legitimately
/// keep Lindera's phonological (katakana) hint, and the residual katakana
/// on words without a dictionary entry belongs to uwuwu#1698.
const ANCHORS: [(&str, &str); 7] = [
    ("平和", "へいわ"),
    ("努力", "どりょく"),
    ("明日", "あした"),
    ("会社員", "かいしゃいん"),
    ("図書館", "としょかん"),
    ("銀行", "ぎんこう"),
    ("月曜日", "げつようび"),
];

fn is_hiragana(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| ('\u{3041}'..='\u{3096}').contains(&c))
}

/// First present corpus entry, or the repo's `[skip]` marker and `None`.
fn corpus_entry(relative: &str, what: &str) -> Option<PathBuf> {
    let path = corpus_roots()
        .into_iter()
        .map(|root| root.join(relative))
        .find(|path| path.is_file());
    if path.is_none() {
        let probe = corpus_roots()
            .first()
            .expect("at least one corpus root")
            .join(relative);
        origa::corpus::print_corpus_skip_marker(what, &probe);
    }
    path
}

/// Deploy window guard: a corpus present but newer than the committed
/// grammar pin is not a stable base for content assertions. Mirrors
/// `origa/tests/common/grammar_corpus.rs` (shared module is crate-local).
fn corpus_pin_drifted(root: &Path, label: &str) -> bool {
    let Some(bytes) = fs::read(root.join(GRAMMAR_SOURCE)).ok() else {
        return false; // absence is the caller's skip, not drift
    };
    let seen = sha256_bytes(&bytes)
        .iter()
        .fold(String::with_capacity(64), |mut out, byte| {
            let _ = write!(out, "{byte:02x}");
            out
        });
    let pinned = GRAMMAR_PIN
        .split_whitespace()
        .next()
        .expect("pin file carries a hash token");
    if seen == pinned {
        return false;
    }
    eprintln!(
        "[skip] {label}: corpus drift — {} sha256 {seen} != pinned {pinned}; \
         skip while the content deploy window is open",
        root.join(GRAMMAR_SOURCE).display()
    );
    true
}

fn assert_anchor_readings(deflated: &[u8], source: &str) {
    let (view, header) =
        access_precompute_deflated(deflated).unwrap_or_else(|e| panic!("{source}: decode: {e:?}"));
    assert_eq!(
        header.schema_version, SCHEMA_VERSION,
        "{source}: schema version"
    );

    let mut seen = BTreeSet::new();
    for (key, entry) in view.entries.iter() {
        for span in entry.furigana_spans.iter() {
            let surface = span.text.as_str();
            let Some((_, expected)) = ANCHORS.iter().find(|(word, _)| *word == surface) else {
                continue;
            };
            seen.insert(surface);
            let reading = span
                .reading
                .as_ref()
                .unwrap_or_else(|| panic!("{source}: {surface} has no reading"))
                .as_str();
            assert_eq!(
                reading,
                *expected,
                "{source}: {surface} (key {:?}) reading",
                key.as_str()
            );
            assert!(
                !span.reading_spans.is_empty(),
                "{source}: {surface} has no per-kanji spans"
            );
            for rs in span.reading_spans.iter() {
                assert!(
                    is_hiragana(rs.text.as_str()),
                    "{source}: {surface} span {:?} is not hiragana",
                    rs.text.as_str()
                );
            }
        }
    }
    for (word, _) in ANCHORS {
        assert!(
            seen.contains(word),
            "{source}: anchor {word} missing from the blob"
        );
    }
}

/// Acceptance at the CDN boundary: the actual delivered bytes must carry
/// hiragana at the same anchors the current annotator produces. A stale
/// blob with an unchanged freshness header fails here even when the
/// manifest guard is valid — exactly the uwuwu#1673 symptom.
#[test]
fn delivered_grammar_blob_renders_anchor_readings_in_hiragana() {
    let path = std::env::var_os("ORIGA_TEST_BLOB")
        .map(PathBuf::from)
        .or_else(|| corpus_entry(GRAMMAR_BLOB, "grammar precompute blob"));
    let Some(path) = path else {
        return;
    };
    // Check the corpus that supplied the bytes; an explicit blob override
    // falls back to the first corpus root carrying the pinned grammar.
    let root = corpus_roots()
        .into_iter()
        .find(|root| path.starts_with(root))
        .or_else(|| {
            corpus_roots()
                .into_iter()
                .find(|root| root.join(GRAMMAR_SOURCE).is_file())
        });
    if root.is_some_and(|root| corpus_pin_drifted(&root, "grammar precompute readings")) {
        return;
    }
    let deflated = fs::read(&path).unwrap_or_else(|e| panic!("reading {}: {e}", path.display()));
    assert_anchor_readings(&deflated, &path.display().to_string());
}

/// Throwaway corpus with Drop cleanup — a red assertion must not leave
/// symlinks into the shared seed behind (same guard as `precompute_skip.rs`).
struct StagedCorpus {
    root: PathBuf,
}

impl StagedCorpus {
    fn new(furigana: &Path, dict: &Path) -> Self {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after the epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("origa-grammar-readings-{unique}"));
        fs::create_dir_all(root.join("grammar")).expect("staging grammar dir");
        fs::create_dir_all(root.join("dictionaries")).expect("staging dictionaries dir");
        fs::write(root.join(GRAMMAR_SOURCE), SAMPLE_RULES).expect("staging grammar source");
        symlink(furigana, &root.join(FURIGANA_SOURCE));
        symlink(dict, &root.join("dictionaries").join(SUDACHIDICT_DIR));
        Self { root }
    }

    fn build(&self) -> std::process::Output {
        Command::new(env!("CARGO_BIN_EXE_utils"))
            .current_dir(&self.root)
            .env("ORIGA_CDN_DIR", &self.root)
            .env("RUST_LOG", "info")
            .arg("build-grammar-precompute")
            .output()
            .unwrap_or_else(|e| panic!("spawning utils builder: {e}"))
    }
}

impl Drop for StagedCorpus {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

/// Unit-to-CLI pin: a fresh build from the real dictionaries produces
/// hiragana at the anchors. Reverting the annotator to the pre-#929
/// katakana conversion — which leaves the freshness header untouched —
/// turns this test red. The repeat-skip behavior is covered by
/// `utils/tests/precompute_skip.rs`; this test owns only the readings.
#[test]
fn forced_rebuild_uses_hiragana_for_dictionary_words() {
    let Some(furigana) = corpus_entry(FURIGANA_SOURCE, "JmdictFurigana for grammar rebuild") else {
        return;
    };
    let Some(dict) = seeded_dictionary_dir() else {
        let probe = corpus_roots()
            .first()
            .expect("at least one corpus root")
            .join("dictionaries")
            .join(SUDACHIDICT_DIR);
        origa::corpus::print_corpus_skip_marker(
            "SudachiDict for grammar precompute rebuild",
            &probe,
        );
        return;
    };

    let corpus = StagedCorpus::new(&furigana, &dict);
    let built = corpus.build();
    let text = output_text(&built);
    assert!(built.status.success(), "{text}");
    assert!(
        text.contains("7 entries derived"),
        "expected the forced build to derive seven entries, got: {text}"
    );
    let blob = corpus.root.join(GRAMMAR_BLOB);
    let rebuilt = fs::read(&blob).expect("builder must write the grammar blob");
    assert_anchor_readings(&rebuilt, "rebuilt blob");
}
