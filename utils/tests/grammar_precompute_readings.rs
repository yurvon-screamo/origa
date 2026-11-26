//! Regression pins for uwuwu#1673: the grammar precompute blob must carry
//! hiragana readings for dictionary-covered kanji words.
//!
//! The blob freshness header binds to `grammar_v3.json`, the SudachiDict
//! directory name and the sha256 of `JmdictFurigana.txt` — not to the
//! annotator code (AGENTS.md / `utils/README.md` freshness boundary).
//! Rebuilding after PR origa#929 was therefore a manual, forced step: the
//! old blob still looked fresh while carrying katakana readings. These
//! tests pin the readings themselves, at two points:
//!
//! 1. the delivered CDN corpus (the bytes CI downloads from the prod
//!    manifest) — integration with the CDN boundary;
//! 2. a forced rebuild through the real CLI with seeded dictionaries —
//!    the builder/annotator path that produces those bytes.
//!
//! Both follow the repo's corpus skip contract: absent corpus -> skip
//! marker and early return; present-but-corrupt -> loud failure.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

use origa::dictionary::cdn_blob::SCHEMA_VERSION;
use origa::dictionary::precompute_blob::access_precompute_deflated;
use origa::domain::SUDACHIDICT_DIR;

const GRAMMAR_BLOB: &str = "grammar/grammar_precompute_v3.rkyv";
const FURIGANA_SOURCE: &str = "dictionaries/JmdictFurigana.txt";
const GRAMMAR_SOURCE: &str = "grammar/grammar_v3.json";
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

/// The corpus root used by tests in CI (`cdn/` artifact) or locally
/// (`$ORIGA_CDN_DIR` / `~/origa-cdn`). `ORIGA_TEST_BLOB` is an explicit
/// override for pointing the delivered-artifact pin at a downloaded copy.
fn corpus_candidates() -> Vec<PathBuf> {
    let project_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("utils crate lives in the workspace root");
    let mut candidates = vec![project_root.join("cdn"), origa::corpus::cdn_dir()];
    candidates.dedup();
    candidates
}

fn corpus_entry(relative: &str, what: &str) -> Option<PathBuf> {
    let path = corpus_candidates()
        .into_iter()
        .map(|root| root.join(relative))
        .find(|path| path.is_file());
    if path.is_none() {
        let probe = corpus_candidates()
            .first()
            .expect("at least one corpus root")
            .join(relative);
        origa::corpus::print_corpus_skip_marker(what, &probe);
    }
    path
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
    let deflated = fs::read(&path).unwrap_or_else(|e| panic!("reading {}: {e}", path.display()));
    assert_anchor_readings(&deflated, &path.display().to_string());
}

fn output_text(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

#[cfg(unix)]
fn link_file(target: &Path, link: &Path) {
    std::os::unix::fs::symlink(target, link).expect("linking seeded dictionary into corpus");
}

#[cfg(windows)]
fn link_file(target: &Path, link: &Path) {
    std::os::windows::fs::symlink_file(target, link)
        .expect("linking seeded dictionary into corpus");
}

#[cfg(unix)]
fn link_dir(target: &Path, link: &Path) {
    std::os::unix::fs::symlink(target, link).expect("linking seeded SudachiDict into corpus");
}

#[cfg(windows)]
fn link_dir(target: &Path, link: &Path) {
    std::os::windows::fs::symlink_dir(target, link)
        .expect("linking seeded SudachiDict into corpus");
}

/// Unit-to-CLI pin: a fresh build from the real dictionaries produces
/// hiragana at the anchors. Reverting the annotator to the pre-#929
/// katakana conversion — which leaves the freshness header untouched —
/// turns this test red. The follow-up run must skip on the unchanged
/// source, as the documented freshness contract promises.
#[test]
fn forced_rebuild_uses_hiragana_for_dictionary_words() {
    let furigana = corpus_entry(FURIGANA_SOURCE, "JmdictFurigana for grammar rebuild");
    let dict = corpus_candidates()
        .into_iter()
        .map(|root| root.join("dictionaries").join(SUDACHIDICT_DIR))
        .find(|dir| dir.join("metadata.json").is_file());
    let (Some(furigana), Some(dict)) = (furigana, dict) else {
        return;
    };

    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock is after the epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("origa-grammar-readings-{unique}"));
    fs::create_dir_all(root.join("grammar")).expect("staging grammar dir");
    fs::create_dir_all(root.join("dictionaries")).expect("staging dictionaries dir");
    fs::write(root.join(GRAMMAR_SOURCE), SAMPLE_RULES).expect("staging grammar source");
    link_file(&furigana, &root.join(FURIGANA_SOURCE));
    link_dir(&dict, &root.join("dictionaries").join(SUDACHIDICT_DIR));

    let build = || {
        Command::new(env!("CARGO_BIN_EXE_utils"))
            .current_dir(&root)
            .env("ORIGA_CDN_DIR", &root)
            .env("RUST_LOG", "info")
            .arg("build-grammar-precompute")
            .output()
            .unwrap_or_else(|e| panic!("spawning utils builder: {e}"))
    };

    let first = build();
    let first_text = output_text(&first);
    assert!(first.status.success(), "{first_text}");
    assert!(
        first_text.contains("7 entries derived"),
        "expected the forced build to derive seven entries, got: {first_text}"
    );
    let blob = root.join(GRAMMAR_BLOB);
    let rebuilt = fs::read(&blob).expect("builder must write the grammar blob");
    assert_anchor_readings(&rebuilt, "rebuilt blob");

    let second = build();
    let second_text = output_text(&second);
    assert!(second.status.success(), "{second_text}");
    assert!(
        second_text.contains("is fresh, skipping regeneration"),
        "expected the freshness skip on repeat, got: {second_text}"
    );
    assert_eq!(rebuilt, fs::read(&blob).expect("blob after repeat"));

    let _ = fs::remove_dir_all(&root);
}
