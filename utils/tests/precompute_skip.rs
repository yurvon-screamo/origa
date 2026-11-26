//! Both precompute builders run on CDN deploy machines that have no local
//! SudachiDict. Fresh blobs must short-circuit before tokenizer init.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

use origa::dictionary::cdn_blob::{BlobHeader, SCHEMA_VERSION, build_blob, sha256_bytes};
use origa::dictionary::precompute_blob::{
    PrecomputeBlob, deflate_blob, serialize_precompute_blob_to_rkyv,
};
use origa::domain::SUDACHIDICT_DIR;

const FURIGANA_SOURCE: &str = "学生 がくせい\n";
const GRAMMAR_SOURCE: &[u8] = br#"{"rules": []}"#;
const PHRASE_CHUNK_SOURCE: &str = r#"[{"x":"私は学生です。"}]"#;
const GRAMMAR_BLOB: &str = "grammar/grammar_precompute_v3.rkyv";
const PHRASE_BLOB: &str = "phrases/precomputed/p0000.rkyv";

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

#[test]
fn deploy_loop_skips_both_builders_without_a_dictionary() {
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
}
