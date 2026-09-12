//! Offline builder for phrase tokenization precompute blobs (#521).
//!
//! For every phrase data chunk (`phrases/data/pNNNN.json`) this derives
//! the tokenization artifacts (furigana spans + tokens) of each phrase
//! text and its [`split_japanese_sentences`] sentences — the exact
//! strings phrase components pass to `FuriganaText`/`TranslatorText` —
//! and writes one deterministic rkyv blob per chunk to
//! `phrases/precomputed/pNNNN.rkyv`.
//!
//! Blob freshness binds to the chunk bytes AND to the tokenizer inputs
//! (SudachiDict version, furigana dictionary source hash): a dictionary
//! bump regenerates every chunk even though the chunk files are
//! unchanged.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use origa::dictionary::cdn_blob::{self, BlobHeader, SCHEMA_VERSION, build_blob, split_blob};
use origa::dictionary::furigana_dict::{
    build_furigana_dict_from_text, serialize_furigana_dict_to_rkyv, set_furigana_dict,
};
use origa::dictionary::precompute_blob::{
    PrecomputeBlob, deflate_blob, inflate_blob, serialize_precompute_blob_to_rkyv,
};
use origa::domain::{
    JapaneseText, OrigaError, PrecomputedEntry, PrecomputedToken, SUDACHIDICT_DIR, annotate_text,
    split_japanese_sentences, tokenize_text,
};
use sha2::{Digest, Sha256};

use crate::dictionary::load_dictionary;

const FURIGANA_SOURCE: &str = "dictionaries/JmdictFurigana.txt";
const PHRASE_DATA_DIR: &str = "phrases/data";
const PRECOMPUTE_DIR: &str = "phrases/precomputed";

fn sha256_hex(data: &[u8]) -> String {
    let digest: [u8; 32] = Sha256::digest(data).into();
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

fn sha256_raw(data: &[u8]) -> [u8; 32] {
    Sha256::digest(data).into()
}

fn chunk_ids(cdn_dir: &Path) -> Result<Vec<u32>, String> {
    let mut ids = Vec::new();
    let entries = fs::read_dir(cdn_dir.join(PHRASE_DATA_DIR))
        .map_err(|e| format!("failed to list {PHRASE_DATA_DIR}: {e}"))?;
    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(stem) = name.to_str().and_then(|n| n.strip_suffix(".json")) else {
            continue;
        };
        let Some(digits) = stem.strip_prefix('p') else {
            continue;
        };
        if let Ok(id) = digits.parse::<u32>() {
            ids.push(id);
        }
    }
    ids.sort_unstable();
    Ok(ids)
}

/// Derives the precompute entry for one exact render string. Kanji-free
/// strings are skipped: the runtime fast path answers them without any
/// dictionary, so precomputing them would only bloat the blob.
fn precompute_entry(text: &str) -> Option<PrecomputedEntry> {
    if !text.contains_kanji() {
        return None;
    }
    let tokens: Vec<PrecomputedToken> = tokenize_text(text)
        .ok()?
        .iter()
        .map(PrecomputedToken::from_token_info)
        .collect();
    let furigana_spans = annotate_text(text).ok()?;
    Some(PrecomputedEntry {
        furigana_spans,
        tokens,
    })
}

fn chunk_entry_keys(text: &str) -> Vec<String> {
    // Full text (lesson TTS reads the whole phrase) plus every sentence
    // the render components split it into.
    let mut keys = vec![text.to_string()];
    for sentence in split_japanese_sentences(text) {
        if sentence != text {
            keys.push(sentence);
        }
    }
    keys
}

fn build_chunk(
    cdn_dir: &Path,
    chunk_id: u32,
    dictionary_ingredients: &[u8],
) -> Result<bool, String> {
    let chunk_path = format!("{PHRASE_DATA_DIR}/p{chunk_id:04}.json");
    let source = fs::read(cdn_dir.join(&chunk_path))
        .map_err(|e| format!("failed to read {chunk_path}: {e}"))?;
    let mut source_with_ingredients = source.clone();
    source_with_ingredients.extend_from_slice(dictionary_ingredients);

    let blob_path = format!("{PRECOMPUTE_DIR}/p{chunk_id:04}.rkyv");
    let existing = fs::read(cdn_dir.join(&blob_path))
        .ok()
        .and_then(|deflated| inflate_blob(&deflated).ok());
    let blob_is_fresh = matches!(existing.as_deref().map(split_blob), Some(Ok((header, _))) if
        header.schema_version == SCHEMA_VERSION
            && header.source_sha256 == sha256_raw(&source_with_ingredients));
    if blob_is_fresh {
        return Ok(false);
    }

    let phrases: Vec<serde_json::Value> = serde_json::from_slice(&source)
        .map_err(|e| format!("failed to parse {chunk_path}: {e}"))?;

    let mut entries: BTreeMap<String, PrecomputedEntry> = BTreeMap::new();
    for phrase in &phrases {
        let Some(text) = phrase.get("x").and_then(|v| v.as_str()) else {
            continue;
        };
        for key in chunk_entry_keys(text) {
            if let Some(entry) = precompute_entry(&key) {
                entries.insert(key, entry);
            }
        }
    }

    let payload = serialize_precompute_blob_to_rkyv(&PrecomputeBlob { entries })
        .map_err(|e| format!("failed to serialize {blob_path}: {e:?}"))?;

    // Guard is derived from the manifest hash of the chunk JSON alone (the
    // only manifest-visible source); the dictionary ingredients only feed
    // the freshness hash above.
    let guard = cdn_blob::manifest_guard_from_hex_hashes(&[&sha256_hex(&source)]);
    let header = BlobHeader {
        schema_version: SCHEMA_VERSION,
        source_sha256: sha256_raw(&source_with_ingredients),
        manifest_guard: guard,
    };
    fs::create_dir_all(cdn_dir.join(PRECOMPUTE_DIR))
        .map_err(|e| format!("failed to create {PRECOMPUTE_DIR}: {e}"))?;
    let built = build_blob(&header, &payload);
    fs::write(cdn_dir.join(&blob_path), deflate_blob(&built))
        .map_err(|e| format!("failed to write {blob_path}: {e}"))?;
    Ok(true)
}

pub fn run_build_phrase_precompute(cdn_dir: Option<&Path>) -> Result<(), OrigaError> {
    let cdn_dir = cdn_dir.unwrap_or_else(|| Path::new("cdn"));
    load_dictionary().map_err(|e| OrigaError::RepositoryError {
        reason: format!("tokenizer dictionary unavailable for precompute: {e:?}"),
    })?;

    let furigana_source =
        fs::read(cdn_dir.join(FURIGANA_SOURCE)).map_err(|e| OrigaError::RepositoryError {
            reason: format!("failed to read {FURIGANA_SOURCE}: {e}"),
        })?;
    let furigana_text =
        String::from_utf8(furigana_source.clone()).map_err(|e| OrigaError::RepositoryError {
            reason: format!("{FURIGANA_SOURCE} is not valid UTF-8: {e}"),
        })?;
    let dict = build_furigana_dict_from_text(&furigana_text)?;
    // Keep a serialized copy out of the loop below: hashing the source is
    // enough for the ingredients, but building asserts the source parses.
    let _ = serialize_furigana_dict_to_rkyv(&dict);
    set_furigana_dict(dict).map_err(|e| OrigaError::RepositoryError {
        reason: format!("furigana dictionary already loaded: {e:?}"),
    })?;

    let mut dictionary_ingredients = Vec::new();
    dictionary_ingredients.extend_from_slice(b"|sudachidict:");
    dictionary_ingredients.extend_from_slice(SUDACHIDICT_DIR.as_bytes());
    dictionary_ingredients.extend_from_slice(b"|furigana:");
    dictionary_ingredients.extend_from_slice(&sha256_raw(&furigana_source));

    let ids = chunk_ids(cdn_dir).map_err(|reason| OrigaError::RepositoryError { reason })?;
    tracing::info!("precomputing {} phrase chunks", ids.len());
    let started = std::time::Instant::now();
    let mut rebuilt = 0usize;
    for chunk_id in ids {
        let did_rebuild = build_chunk(cdn_dir, chunk_id, &dictionary_ingredients)
            .map_err(|reason| OrigaError::RepositoryError { reason })?;
        rebuilt += usize::from(did_rebuild);
    }
    tracing::info!(
        "phrase precompute done: {rebuilt} chunks rebuilt in {:.1}s",
        started.elapsed().as_secs_f32()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunk_entry_keys_cover_full_text_and_sentences() {
        let keys = chunk_entry_keys("私は学生です。小林さんは先生です。");

        assert_eq!(
            keys,
            vec![
                "私は学生です。小林さんは先生です。".to_string(),
                "私は学生です。".to_string(),
                "小林さんは先生です。".to_string(),
            ]
        );
    }

    #[test]
    fn chunk_entry_keys_dedupe_single_sentence_text() {
        let keys = chunk_entry_keys("こんにちは。");

        assert_eq!(keys, vec!["こんにちは。".to_string()]);
    }
}
