//! The grammar precompute builder runs on CDN deploy machines that have no
//! local SudachiDict. A fresh blob must short-circuit before tokenizer init.

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

use origa::dictionary::cdn_blob::{BlobHeader, SCHEMA_VERSION, build_blob, sha256_bytes};
use origa::dictionary::precompute_blob::{
    PrecomputeBlob, deflate_blob, serialize_precompute_blob_to_rkyv,
};
use origa::domain::SUDACHIDICT_DIR;

const FURIGANA_SOURCE: &str = "学生 がくせい\n";
const GRAMMAR_SOURCE: &[u8] = br#"{"rules": []}"#;
const BLOB_PATH: &str = "grammar/grammar_precompute_v3.rkyv";

struct Corpus {
    root: PathBuf,
}

impl Corpus {
    /// Stages a corpus with the grammar and furigana sources and a blob
    /// whose source hash matches them when `fresh` is true. No SudachiDict
    /// and no `target/cdn-cache` are staged — the tokenizer is unavailable.
    fn staged(fresh: bool) -> Self {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after the epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "origa-grammar-precompute-skip-{}-{unique}",
            std::process::id()
        ));
        fs::create_dir_all(root.join("grammar")).expect("staging grammar dir");
        fs::create_dir_all(root.join("dictionaries")).expect("staging dictionaries dir");
        fs::write(root.join("grammar/grammar_v3.json"), GRAMMAR_SOURCE)
            .expect("staging grammar source");
        fs::write(
            root.join("dictionaries/JmdictFurigana.txt"),
            FURIGANA_SOURCE,
        )
        .expect("staging furigana source");

        let mut ingredients = format!("|sudachidict:{SUDACHIDICT_DIR}|furigana:").into_bytes();
        ingredients.extend_from_slice(&sha256_bytes(FURIGANA_SOURCE.as_bytes()));
        let mut source_with_ingredients = GRAMMAR_SOURCE.to_vec();
        source_with_ingredients.extend_from_slice(&ingredients);
        let digest = if fresh {
            sha256_bytes(&source_with_ingredients)
        } else {
            [0u8; 32]
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
        fs::write(
            root.join(BLOB_PATH),
            deflate_blob(&build_blob(&header, &payload)),
        )
        .expect("staging precompute blob");

        Self { root }
    }

    fn build(&self) -> Output {
        Command::new(env!("CARGO_BIN_EXE_utils"))
            .current_dir(&self.root)
            .env("ORIGA_CDN_DIR", &self.root)
            .env("RUST_LOG", "info")
            .arg("build-grammar-precompute")
            .output()
            .expect("spawning utils build-grammar-precompute")
    }
}

impl Drop for Corpus {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn output_text(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

#[test]
fn fresh_blob_skips_before_loading_the_tokenizer() {
    let corpus = Corpus::staged(true);

    let output = corpus.build();

    assert!(output.status.success(), "{}", output_text(&output));
    assert!(
        output_text(&output).contains("grammar_precompute_v3.rkyv is fresh, skipping regeneration"),
        "expected the skip log, got: {}",
        output_text(&output)
    );
}

#[test]
fn stale_blob_still_attempts_regeneration() {
    let corpus = Corpus::staged(false);

    let output = corpus.build();

    assert!(!output.status.success(), "{}", output_text(&output));
    assert!(
        output_text(&output).contains("tokenizer dictionary unavailable for precompute"),
        "expected regeneration to load the tokenizer, got: {}",
        output_text(&output)
    );
}
