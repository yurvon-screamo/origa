//! Both precompute builders run on CDN deploy machines that have no local
//! SudachiDict. Fresh blobs must short-circuit before tokenizer init.
//!
//! Prerequisites for this suite: `cargo` and `python3` in PATH. The positive
//! path needs a seeded SudachiDict — in CI the `cdn-data` artifact is
//! unpacked to `<workspace>/cdn`, locally it is `$ORIGA_CDN_DIR` or
//! `~/origa-cdn`; without one those tests exit early with the repo's
//! `[skip]` corpus marker.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

use origa::dictionary::cdn_blob::{BlobHeader, SCHEMA_VERSION, build_blob, sha256_bytes};
use origa::dictionary::precompute_blob::{
    PrecomputeBlob, access_precompute_deflated, deflate_blob, serialize_precompute_blob_to_rkyv,
};
use origa::domain::SUDACHIDICT_DIR;

const FURIGANA_SOURCE: &str = "学生 がくせい\n";
// A kanji-bearing rule: the builder must derive a real entry, not an empty map.
const GRAMMAR_SOURCE: &[u8] = b"{\"rules\":[{\"title\":\"\xE5\xAD\xA6\xE7\x94\x9F\"}]}";
const PHRASE_CHUNK_SOURCE: &str = r#"[{"x":"私は学生です。"}]"#;
const SECOND_CHUNK_SOURCE: &str = r#"[{"x":"猫が好きです。"}]"#;
const GRAMMAR_BLOB: &str = "grammar/grammar_precompute_v3.rkyv";
const PHRASE_BLOB: &str = "phrases/precomputed/p0000.rkyv";
const SECOND_PHRASE_BLOB: &str = "phrases/precomputed/p0001.rkyv";

#[derive(Clone, Copy)]
enum BlobState {
    Fresh,
    Stale,
}

struct Corpus {
    root: PathBuf,
}

impl Corpus {
    /// Stages grammar and phrase sources plus their blobs. No SudachiDict
    /// and no `target/cdn-cache` are staged — the tokenizer is unavailable.
    fn staged(state: BlobState) -> Self {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after the epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "origa-precompute-skip-{}-{unique}",
            std::process::id()
        ));
        fs::create_dir_all(root.join("grammar")).expect("staging grammar dir");
        fs::create_dir_all(root.join("dictionaries")).expect("staging dictionaries dir");
        fs::create_dir_all(root.join("phrases/data")).expect("staging phrase data dir");
        fs::write(root.join("grammar/grammar_v3.json"), GRAMMAR_SOURCE)
            .expect("staging grammar source");
        fs::write(
            root.join("dictionaries/JmdictFurigana.txt"),
            FURIGANA_SOURCE,
        )
        .expect("staging furigana source");
        fs::write(
            root.join("phrases/data/p0000.json"),
            PHRASE_CHUNK_SOURCE.as_bytes(),
        )
        .expect("staging phrase chunk source");

        let ingredients = dictionary_ingredients();
        stage_blob(&root, GRAMMAR_BLOB, GRAMMAR_SOURCE, &ingredients, state);
        stage_blob(
            &root,
            PHRASE_BLOB,
            PHRASE_CHUNK_SOURCE.as_bytes(),
            &ingredients,
            state,
        );
        Self { root }
    }

    /// Symlinks the seeded SudachiDict into the corpus so the regeneration
    /// path can run. False means no seed exists — the repo's skip marker is
    /// emitted and the caller returns early (CI unpacks the dict into cdn/).
    fn link_dictionary(&self) -> bool {
        let Some(dict) = seeded_dictionary_dir() else {
            let probe = origa::corpus::cdn_dir()
                .join("dictionaries")
                .join(SUDACHIDICT_DIR);
            origa::corpus::print_corpus_skip_marker(
                "SudachiDict for precompute regeneration",
                &probe,
            );
            return false;
        };
        link_seed_dir(&dict, &self.root.join("dictionaries").join(SUDACHIDICT_DIR));
        true
    }

    /// Adds a second chunk with a genuinely fresh blob, to prove that only
    /// stale chunks are selected for regeneration.
    fn add_fresh_phrase_chunk(&self, id: u32, source: &[u8]) {
        fs::write(
            self.root.join(format!("phrases/data/p{id:04}.json")),
            source,
        )
        .expect("staging second phrase chunk");
        stage_blob(
            &self.root,
            &format!("phrases/precomputed/p{id:04}.rkyv"),
            source,
            &dictionary_ingredients(),
            BlobState::Fresh,
        );
    }

    fn build(&self, command: &str) -> Output {
        Command::new(env!("CARGO_BIN_EXE_utils"))
            .current_dir(&self.root)
            .env("ORIGA_CDN_DIR", &self.root)
            .env("RUST_LOG", "info")
            .arg(command)
            .output()
            .unwrap_or_else(|e| panic!("spawning utils {command}: {e}"))
    }
}

#[cfg(unix)]
fn link_seed_dir(target: &Path, link: &Path) {
    std::os::unix::fs::symlink(target, link).expect("linking seeded SudachiDict into corpus");
}

#[cfg(windows)]
fn link_seed_dir(target: &Path, link: &Path) {
    std::os::windows::fs::symlink_dir(target, link)
        .expect("linking seeded SudachiDict into corpus");
}

/// The CI Test job unpacks the `cdn-data` artifact into `<workspace>/cdn`;
/// developer machines keep one seed at `$ORIGA_CDN_DIR` / `~/origa-cdn`.
fn seeded_dictionary_dir() -> Option<PathBuf> {
    let project_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent()?;
    let candidates = [
        project_root
            .join("cdn")
            .join("dictionaries")
            .join(SUDACHIDICT_DIR),
        origa::corpus::cdn_dir()
            .join("dictionaries")
            .join(SUDACHIDICT_DIR),
    ];
    candidates
        .into_iter()
        .find(|dir| dir.join("metadata.json").is_file())
}

impl Drop for Corpus {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

/// Mirrors `precompute_common::init_furigana_and_dictionary_ingredients`
/// and `deploy_cdn.py::grammar_precompute_source` (the canonical carriers).
fn dictionary_ingredients() -> Vec<u8> {
    let mut ingredients = format!("|sudachidict:{SUDACHIDICT_DIR}|furigana:").into_bytes();
    ingredients.extend_from_slice(&sha256_bytes(FURIGANA_SOURCE.as_bytes()));
    ingredients
}

fn stage_blob(root: &Path, blob_path: &str, source: &[u8], ingredients: &[u8], state: BlobState) {
    let mut source_with_ingredients = source.to_vec();
    source_with_ingredients.extend_from_slice(ingredients);
    let digest = match state {
        BlobState::Fresh => sha256_bytes(&source_with_ingredients),
        BlobState::Stale => [0u8; 32],
    };
    let header = BlobHeader {
        schema_version: SCHEMA_VERSION,
        source_sha256: digest,
        manifest_guard: [0u8; 32],
    };
    let payload = serialize_precompute_blob_to_rkyv(&PrecomputeBlob {
        entries: BTreeMap::new(),
    })
    .expect("serializing empty precompute blob");
    let path = root.join(blob_path);
    fs::create_dir_all(path.parent().expect("blob path has a parent")).expect("staging blob dir");
    fs::write(path, deflate_blob(&build_blob(&header, &payload))).expect("staging precompute blob");
}

fn output_text(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

/// Decode the rebuilt CDN blob and check that the known key carries real
/// tokenizer output. The exact reading is not pinned (the builder returns
/// katakana for a hiragana fixture) — only non-empty tokens and furigana
/// spans, so a builder that writes an empty map cannot pass.
fn assert_rebuilt_entry(blob: &Path, key: &str) {
    let deflated = fs::read(blob).unwrap_or_else(|e| panic!("reading {}: {e}", blob.display()));
    let (view, _) = access_precompute_deflated(&deflated)
        .unwrap_or_else(|e| panic!("decoding {}: {e:?}", blob.display()));
    let entry = view
        .entries
        .get(key)
        .unwrap_or_else(|| panic!("{key} missing in {}", blob.display()));
    assert!(!entry.tokens.is_empty(), "{key}: no tokens in rebuilt blob");
    assert!(
        entry
            .tokens
            .iter()
            .any(|token| !token.surface.as_str().is_empty() && !token.reading.as_str().is_empty()),
        "{key}: tokens carry no surface/reading"
    );
    assert!(
        !entry.furigana_spans.is_empty(),
        "{key}: no furigana spans in rebuilt blob"
    );
}

#[test]
fn fresh_grammar_blob_skips_before_loading_the_tokenizer() {
    let corpus = Corpus::staged(BlobState::Fresh);

    let output = corpus.build("build-grammar-precompute");

    assert!(output.status.success(), "{}", output_text(&output));
    assert!(
        output_text(&output).contains("grammar_precompute_v3.rkyv is fresh, skipping regeneration"),
        "expected the skip log, got: {}",
        output_text(&output)
    );
}

#[test]
fn stale_grammar_blob_still_attempts_regeneration() {
    let corpus = Corpus::staged(BlobState::Stale);

    let output = corpus.build("build-grammar-precompute");

    assert!(!output.status.success(), "{}", output_text(&output));
    assert!(
        output_text(&output).contains("tokenizer dictionary unavailable for precompute"),
        "expected regeneration to load the tokenizer, got: {}",
        output_text(&output)
    );
}

#[test]
fn fresh_phrase_chunks_skip_before_loading_the_tokenizer() {
    let corpus = Corpus::staged(BlobState::Fresh);

    let output = corpus.build("build-phrase-precompute");

    assert!(output.status.success(), "{}", output_text(&output));
    assert!(
        output_text(&output)
            .contains("all phrase precompute chunks are fresh, skipping regeneration"),
        "expected the skip log, got: {}",
        output_text(&output)
    );
}

#[test]
fn stale_phrase_chunk_still_attempts_regeneration() {
    let corpus = Corpus::staged(BlobState::Stale);

    let output = corpus.build("build-phrase-precompute");

    assert!(!output.status.success(), "{}", output_text(&output));
    assert!(
        output_text(&output).contains("tokenizer dictionary unavailable for precompute"),
        "expected regeneration to load the tokenizer, got: {}",
        output_text(&output)
    );
}

/// Cheap CLI-level check that the two commands can run back to back with no
/// dictionary. The real production caller is driven by the sibling
/// `real_deploy_loop_…` test below.
#[test]
fn both_builders_skip_when_run_sequentially_without_a_dictionary() {
    let corpus = Corpus::staged(BlobState::Fresh);

    let grammar = corpus.build("build-grammar-precompute");
    let phrase = corpus.build("build-phrase-precompute");

    assert!(grammar.status.success(), "{}", output_text(&grammar));
    assert!(phrase.status.success(), "{}", output_text(&phrase));
}

/// Acceptance level: drive the real production caller, not a hand-rolled
/// copy of its loop. `deploy_cdn.py::regenerate_precompute_blobs` runs both
/// builders via `cargo run` and exits on the first non-zero code — the exact
/// Step 0.5 flow that motivated #926. The corpus has no SudachiDict, so a
/// regression that loads the tokenizer before the freshness check turns
/// this red.
#[test]
fn real_deploy_loop_skips_both_builders_without_a_dictionary() {
    let corpus = Corpus::staged(BlobState::Fresh);
    let project_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("utils crate lives in the workspace root");
    let script = concat!(
        "from pathlib import Path;",
        "import deploy_cdn;",
        "deploy_cdn.regenerate_precompute_blobs(Path.cwd());",
        "print('DEPLOY LOOP OK')"
    );

    let output = Command::new("python3")
        .current_dir(project_root)
        .env("PYTHONPATH", project_root.join("scripts"))
        .env("ORIGA_CDN_DIR", &corpus.root)
        .env("RUST_LOG", "info")
        .args(["-c", script])
        .output()
        .unwrap_or_else(|e| panic!("spawning python3 deploy loop: {e}"));

    let text = output_text(&output);
    assert!(output.status.success(), "{text}");
    assert!(
        text.contains("grammar/grammar_precompute_v3.rkyv is fresh, skipping regeneration"),
        "expected grammar skip log, got: {text}"
    );
    assert!(
        text.contains("all phrase precompute chunks are fresh, skipping regeneration"),
        "expected phrase skip log, got: {text}"
    );
    assert!(
        text.contains("DEPLOY LOOP OK"),
        "expected loop completion, got: {text}"
    );
    // The corpus must be dictionary-free: cwd is the checkout, where a
    // stale `target/cdn-cache` would silently mask the skip path.
    assert!(
        !text.contains("using cached CDN dictionary"),
        "the loop must not fall back to target/cdn-cache, got: {text}"
    );
}

/// Acceptance criterion 2: a stale grammar blob is rebuilt by the real
/// tokenizer into a non-empty entry, and then recognised as fresh.
#[test]
fn stale_grammar_blob_is_regenerated_then_skipped() {
    let corpus = Corpus::staged(BlobState::Stale);
    if !corpus.link_dictionary() {
        return;
    }
    let blob = corpus.root.join(GRAMMAR_BLOB);
    let staged = fs::read(&blob).expect("staged stale blob");

    let rebuilt = corpus.build("build-grammar-precompute");
    let rebuilt_text = output_text(&rebuilt);
    assert!(rebuilt.status.success(), "{rebuilt_text}");
    assert!(
        rebuilt_text.contains("1 entries derived"),
        "expected a real entry to be derived, got: {rebuilt_text}"
    );
    let regenerated = fs::read(&blob).expect("regenerated blob");
    assert_ne!(staged, regenerated, "regeneration must overwrite the blob");
    assert_rebuilt_entry(&blob, "学生");

    let again = corpus.build("build-grammar-precompute");
    let again_text = output_text(&again);
    assert!(again.status.success(), "{again_text}");
    assert!(
        again_text.contains("grammar/grammar_precompute_v3.rkyv is fresh, skipping regeneration"),
        "expected the fresh skip on repeat, got: {again_text}"
    );
    assert_eq!(regenerated, fs::read(&blob).expect("blob after repeat"));
}

/// The phrase builder must select only stale chunks: a fresh sibling blob
/// survives byte-for-byte, and the next run skips both.
#[test]
fn only_stale_phrase_chunks_are_rebuilt_then_skipped() {
    let corpus = Corpus::staged(BlobState::Stale);
    corpus.add_fresh_phrase_chunk(1, SECOND_CHUNK_SOURCE.as_bytes());
    if !corpus.link_dictionary() {
        return;
    }
    let stale = corpus.root.join(PHRASE_BLOB);
    let fresh = corpus.root.join(SECOND_PHRASE_BLOB);
    let stale_before = fs::read(&stale).expect("staged stale blob");
    let fresh_before = fs::read(&fresh).expect("staged fresh blob");

    let rebuilt = corpus.build("build-phrase-precompute");
    let rebuilt_text = output_text(&rebuilt);
    assert!(rebuilt.status.success(), "{rebuilt_text}");
    assert!(
        rebuilt_text.contains("precomputing 1 stale phrase chunks")
            && rebuilt_text.contains("1 chunks rebuilt"),
        "expected exactly one chunk to be rebuilt, got: {rebuilt_text}"
    );
    let stale_after = fs::read(&stale).expect("rebuilt blob");
    assert_ne!(stale_before, stale_after, "stale blob must be overwritten");
    assert_rebuilt_entry(&stale, "私は学生です。");
    assert_eq!(
        fresh_before,
        fs::read(&fresh).expect("fresh blob"),
        "fresh chunk must not be rewritten"
    );

    let again = corpus.build("build-phrase-precompute");
    let again_text = output_text(&again);
    assert!(again.status.success(), "{again_text}");
    assert!(
        again_text.contains("all phrase precompute chunks are fresh, skipping regeneration"),
        "expected the fresh skip on repeat, got: {again_text}"
    );
    assert_eq!(stale_after, fs::read(&stale).expect("blob after repeat"));
    assert_eq!(
        fresh_before,
        fs::read(&fresh).expect("fresh blob after repeat")
    );
}

/// Primary build: a missing blob is stale too, and the created blob is
/// recognised as fresh on the next run.
#[test]
fn missing_phrase_blob_is_built_then_skipped() {
    let corpus = Corpus::staged(BlobState::Fresh);
    if !corpus.link_dictionary() {
        return;
    }
    let blob = corpus.root.join(PHRASE_BLOB);
    fs::remove_file(&blob).expect("removing the blob to force a primary build");

    let built = corpus.build("build-phrase-precompute");
    let built_text = output_text(&built);
    assert!(built.status.success(), "{built_text}");
    assert!(
        built_text.contains("1 chunks rebuilt"),
        "expected a primary build, got: {built_text}"
    );
    let created = fs::read(&blob).expect("primary build must create the blob");
    assert_rebuilt_entry(&blob, "私は学生です。");

    let again = corpus.build("build-phrase-precompute");
    let again_text = output_text(&again);
    assert!(again.status.success(), "{again_text}");
    assert!(
        again_text.contains("all phrase precompute chunks are fresh, skipping regeneration"),
        "expected the fresh skip on repeat, got: {again_text}"
    );
    assert_eq!(created, fs::read(&blob).expect("blob after repeat"));
}
