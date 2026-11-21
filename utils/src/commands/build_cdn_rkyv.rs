//! Offline builder for pre-parsed rkyv blobs deployed to the CDN.
//!
//! Produces `dictionaries/JmdictFurigana.rkyv` and `dictionary/vocabulary.rkyv`
//! from the editable sources (`JmdictFurigana.txt`, `chunk_01..11.json`).
//! Clients load these blobs directly instead of parsing text/JSON on every
//! cold start; the sources stay on the CDN as fallback and source-of-truth.
//!
//! Blobs embed `source_sha256` and a `manifest_guard` (derived from the same
//! per-file SHA-256 hex hashes the deploy script puts into the CDN manifest),
//! so clients can verify a blob against the manifest without downloading the
//! sources.

use std::fs;
use std::path::{Path, PathBuf};

use origa::dictionary::cdn_blob::{self, BlobHeader, SCHEMA_VERSION, build_blob, split_blob};
use origa::dictionary::furigana_dict::{
    build_furigana_dict_from_text, serialize_furigana_dict_to_rkyv,
};
use origa::dictionary::phrase::{
    build_phrase_index_from_json, serialize_phrase_index_blob_to_rkyv,
    serialize_phrase_index_v3_blob_to_rkyv,
};
use origa::dictionary::pitch_audio::serialize_pitch_index_blob_to_rkyv;
use origa::dictionary::precompute_blob::{deflate_blob, inflate_blob};
use origa::dictionary::vocabulary::{
    VocabularyChunkData, build_vocabulary_database_from_chunks, serialize_vocabulary_blob_to_rkyv,
};
use origa::domain::OrigaError;
use sha2::{Digest, Sha256};

const FURIGANA_SOURCE: &str = "dictionaries/JmdictFurigana.txt";
const FURIGANA_BLOB: &str = "dictionaries/JmdictFurigana.rkyv";
const VOCABULARY_BLOB: &str = "dictionary/vocabulary.rkyv";
const PHRASE_INDEX_SOURCE: &str = "phrases/phrase_index.json";
const PHRASE_INDEX_BLOB: &str = "phrases/phrase_index.rkyv";
const PITCH_INDEX_SOURCE: &str = "pitch/index.json";
const PITCH_INDEX_BLOB: &str = "pitch/index.rkyv";
const CHUNK_COUNT: usize = 11;

fn sha256_hex(data: &[u8]) -> String {
    let digest: [u8; 32] = Sha256::digest(data).into();
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

fn sha256_raw(data: &[u8]) -> [u8; 32] {
    Sha256::digest(data).into()
}

fn chunk_path(index: usize) -> String {
    format!("dictionary/chunk_{index:02}.json")
}

fn read(cdn_dir: &Path, relative: &str) -> Result<Vec<u8>, String> {
    let path = cdn_dir.join(relative);
    fs::read(&path).map_err(|e| format!("failed to read {}: {e}", path.display()))
}

fn write(cdn_dir: &Path, relative: &str, bytes: &[u8]) -> Result<(), String> {
    let path = cdn_dir.join(relative);
    fs::write(&path, bytes).map_err(|e| format!("failed to write {}: {e}", path.display()))
}

/// Existing blob is skipped only when its header is fresh: same schema
/// version and same source hash as the sources on disk.
fn blob_is_fresh(existing: Option<&[u8]>, schema_version: u32, source_sha256: &[u8; 32]) -> bool {
    match existing.map(split_blob) {
        Some(Ok((header, _))) => {
            header.schema_version == schema_version && header.source_sha256 == *source_sha256
        },
        _ => false,
    }
}

fn deploy_blob(
    cdn_dir: &Path,
    blob_path: &str,
    source_sha256: [u8; 32],
    manifest_guard: [u8; 32],
    payload: Vec<u8>,
) -> Result<bool, String> {
    let existing = read(cdn_dir, blob_path).ok();
    if blob_is_fresh(existing.as_deref(), SCHEMA_VERSION, &source_sha256) {
        tracing::info!("{} is fresh, skipping regeneration", blob_path);
        return Ok(false);
    }

    let header = BlobHeader {
        schema_version: SCHEMA_VERSION,
        source_sha256,
        manifest_guard,
    };
    write(cdn_dir, blob_path, &build_blob(&header, &payload))?;
    tracing::info!("{} written ({} bytes payload)", blob_path, payload.len());
    Ok(true)
}

fn build_furigana(cdn_dir: &Path) -> Result<bool, String> {
    let source = read(cdn_dir, FURIGANA_SOURCE)?;
    let source_hex = sha256_hex(&source);

    // Freshness check BEFORE the expensive parse+serialize: a fresh blob
    // costs one file read, not a full 12 MB text re-parse.
    if blob_is_fresh(
        read(cdn_dir, FURIGANA_BLOB).ok().as_deref(),
        SCHEMA_VERSION,
        &sha256_raw(&source),
    ) {
        tracing::info!("{} is fresh, skipping regeneration", FURIGANA_BLOB);
        return Ok(false);
    }

    let text = String::from_utf8(source.clone())
        .map_err(|e| format!("{FURIGANA_SOURCE} is not valid UTF-8: {e}"))?;
    let dict = build_furigana_dict_from_text(&text)
        .map_err(|e| format!("failed to build furigana dictionary: {e}"))?;
    let payload = serialize_furigana_dict_to_rkyv(&dict)
        .map_err(|e| format!("failed to serialize furigana dictionary: {e}"))?;

    let guard = cdn_blob::manifest_guard_from_hex_hashes(&[&source_hex]);
    deploy_blob(cdn_dir, FURIGANA_BLOB, sha256_raw(&source), guard, payload)
}

fn build_vocabulary(cdn_dir: &Path) -> Result<bool, String> {
    let mut chunks: Vec<(String, Vec<u8>)> = Vec::with_capacity(CHUNK_COUNT);
    let mut concatenated = Vec::new();
    for index in 1..=CHUNK_COUNT {
        let path = chunk_path(index);
        let bytes = read(cdn_dir, &path)?;
        concatenated.extend_from_slice(&bytes);
        chunks.push((path, bytes));
    }

    // Freshness check BEFORE the expensive JSON parse + rkyv serialize:
    // chunk reads are cheap, the parse is not.
    if blob_is_fresh(
        read(cdn_dir, VOCABULARY_BLOB).ok().as_deref(),
        SCHEMA_VERSION,
        &sha256_raw(&concatenated),
    ) {
        tracing::info!("{} is fresh, skipping regeneration", VOCABULARY_BLOB);
        return Ok(false);
    }

    let chunk_data = assemble_chunk_data(&chunks)?;

    let db = build_vocabulary_database_from_chunks(chunk_data)
        .map_err(|e| format!("failed to build vocabulary database: {e}"))?;
    let payload = serialize_vocabulary_blob_to_rkyv(&db)
        .map_err(|e| format!("failed to serialize vocabulary blob: {e}"))?;

    let hex_hashes: Vec<String> = chunks.iter().map(|(_, b)| sha256_hex(b)).collect();
    let hex_refs: Vec<&str> = hex_hashes.iter().map(String::as_str).collect();
    let guard = cdn_blob::manifest_guard_from_hex_hashes(&hex_refs);

    deploy_blob(
        cdn_dir,
        VOCABULARY_BLOB,
        sha256_raw(&concatenated),
        guard,
        payload,
    )
}

fn assemble_chunk_data(chunks: &[(String, Vec<u8>)]) -> Result<VocabularyChunkData, String> {
    let parse = |index: usize| -> Result<String, String> {
        let bytes = &chunks[index - 1].1;
        String::from_utf8(bytes.clone())
            .map_err(|e| format!("{} is not valid UTF-8: {e}", chunk_path(index)))
    };

    Ok(VocabularyChunkData {
        chunk_01: parse(1)?,
        chunk_02: parse(2)?,
        chunk_03: parse(3)?,
        chunk_04: parse(4)?,
        chunk_05: parse(5)?,
        chunk_06: parse(6)?,
        chunk_07: parse(7)?,
        chunk_08: parse(8)?,
        chunk_09: parse(9)?,
        chunk_10: parse(10)?,
        chunk_11: parse(11)?,
    })
}

/// Build a single-source blob (phrase/pitch pattern): parse the JSON
/// source, serialize the deterministic blob form, verify the manifest
/// guard. Freshness is checked before the expensive parse.
fn build_single_source_index(
    cdn_dir: &Path,
    source_path: &str,
    blob_path: &str,
    build: impl Fn(&str) -> Result<Vec<u8>, String>,
) -> Result<bool, String> {
    let source = read(cdn_dir, source_path)?;
    let source_hex = sha256_hex(&source);

    // Freshness check BEFORE the expensive parse + serialize.
    if blob_is_fresh(
        read(cdn_dir, blob_path).ok().as_deref(),
        SCHEMA_VERSION,
        &sha256_raw(&source),
    ) {
        tracing::info!("{blob_path} is fresh, skipping regeneration");
        return Ok(false);
    }

    let text = String::from_utf8(source.clone())
        .map_err(|e| format!("{source_path} is not valid UTF-8: {e}"))?;
    let payload = build(&text)?;

    let guard = cdn_blob::manifest_guard_from_hex_hashes(&[&source_hex]);
    deploy_blob(cdn_dir, blob_path, sha256_raw(&source), guard, payload)
}

fn build_phrase_index_blob(cdn_dir: &Path) -> Result<bool, String> {
    build_single_source_index(cdn_dir, PHRASE_INDEX_SOURCE, PHRASE_INDEX_BLOB, |text| {
        let index = build_phrase_index_from_json(text)
            .map_err(|e| format!("failed to build phrase index: {e}"))?;
        serialize_phrase_index_blob_to_rkyv(&index.to_blob())
            .map_err(|e| format!("failed to serialize phrase index blob: {e}"))
    })
}

/// #535 — the deduplicated + interned v3 phrase index blob. Stored
/// deflate-compressed on the CDN (the payload is string-heavy); the
/// freshness read therefore inflates before the header check. The v2
/// blob keeps building alongside (old clients stay on it until the
/// minimum supported version rises).
fn build_phrase_index_v3_blob(cdn_dir: &Path) -> Result<bool, String> {
    const PHRASE_INDEX_BLOB_V3: &str = "phrases/phrase_index.v3.rkyv";

    let source = read(cdn_dir, PHRASE_INDEX_SOURCE)?;
    let source_hex = sha256_hex(&source);
    let source_hash = sha256_raw(&source);

    let existing = read(cdn_dir, PHRASE_INDEX_BLOB_V3)
        .ok()
        .and_then(|deflated| inflate_blob(&deflated).ok());
    let is_fresh = matches!(
        existing.as_deref().map(split_blob),
        Some(Ok((header, _))) if header.schema_version == SCHEMA_VERSION
            && header.source_sha256 == source_hash
    );
    if is_fresh {
        tracing::info!("{PHRASE_INDEX_BLOB_V3} is fresh, skipping regeneration");
        return Ok(false);
    }

    let text = String::from_utf8(source.clone())
        .map_err(|e| format!("{PHRASE_INDEX_SOURCE} is not valid UTF-8: {e}"))?;
    let index = build_phrase_index_from_json(&text)
        .map_err(|e| format!("failed to build phrase index: {e}"))?;
    let payload = serialize_phrase_index_v3_blob_to_rkyv(&index.to_blob_v3())
        .map_err(|e| format!("failed to serialize phrase index v3 blob: {e}"))?;

    let guard = cdn_blob::manifest_guard_from_hex_hashes(&[&source_hex]);
    let header = BlobHeader {
        schema_version: SCHEMA_VERSION,
        source_sha256: source_hash,
        manifest_guard: guard,
    };
    let built = build_blob(&header, &payload);
    let deflated = deflate_blob(&built);
    let deflated_len = deflated.len();
    fs::write(cdn_dir.join(PHRASE_INDEX_BLOB_V3), deflated)
        .map_err(|e| format!("failed to write {PHRASE_INDEX_BLOB_V3}: {e}"))?;
    tracing::info!(
        "rebuilt {PHRASE_INDEX_BLOB_V3}: {:.1} MB deflated ({:.1} MB raw payload)",
        deflated_len as f64 / 1024.0 / 1024.0,
        payload.len() as f64 / 1024.0 / 1024.0
    );
    Ok(true)
}

/// Number of phrase data bundles on the CDN — keep in sync with
/// scripts/bundle_phrases_data.py (CHUNKS_PER_BUNDLE = 34, 198 chunks).
const PHRASE_BUNDLE_COUNT: usize = 6;

fn phrase_bundle_source_path(index: usize) -> String {
    format!("phrases/data_bundle_{index}.json")
}

fn phrase_bundle_blob_path(index: usize) -> String {
    format!("phrases/data_bundle_{index}.v3.rkyv")
}

/// Deflated bundle blobs (uwuwu #1451): precache downloads these instead of
/// raw JSON bundles (~5x smaller on the wire). The payload is the
/// deterministic bundle JSON itself — client extraction parses generic JSON
/// and splits it into per-chunk cache entries. The manifest guard binds the
/// blob to the chunk files (phrases/data/pXXXX.json) the bundle was
/// assembled from, so a deploy-window bundle is rejected client-side.
fn build_phrase_bundle_blobs(cdn_dir: &Path) -> Result<bool, String> {
    let mut rebuilt = false;
    for index in 0..PHRASE_BUNDLE_COUNT {
        rebuilt |= build_phrase_bundle_blob(cdn_dir, index)?;
    }
    Ok(rebuilt)
}

fn build_phrase_bundle_blob(cdn_dir: &Path, index: usize) -> Result<bool, String> {
    let source_path = phrase_bundle_source_path(index);
    let blob_path = phrase_bundle_blob_path(index);

    let source = read(cdn_dir, &source_path)?;
    let source_hash = sha256_raw(&source);

    // Freshness check BEFORE the guard derivation: a fresh blob costs one
    // read + inflate, not a re-read of its 34 chunk files.
    let existing = read(cdn_dir, &blob_path)
        .ok()
        .and_then(|deflated| inflate_blob(&deflated).ok());
    let is_fresh = matches!(
        existing.as_deref().map(split_blob),
        Some(Ok((header, _))) if header.schema_version == SCHEMA_VERSION
            && header.source_sha256 == source_hash
    );
    if is_fresh {
        tracing::info!("{blob_path} is fresh, skipping regeneration");
        return Ok(false);
    }

    let manifest_guard = phrase_bundle_manifest_guard(cdn_dir, &source)?;
    let header = BlobHeader {
        schema_version: SCHEMA_VERSION,
        source_sha256: source_hash,
        manifest_guard,
    };
    let built = build_blob(&header, &source);
    let deflated = deflate_blob(&built);
    let deflated_len = deflated.len();
    write(cdn_dir, &blob_path, &deflated)?;
    tracing::info!(
        "rebuilt {blob_path}: {:.1} MB deflated",
        deflated_len as f64 / 1024.0 / 1024.0
    );
    Ok(true)
}

/// Guard over the chunk files the bundle contains: the bundle JSON keys
/// (sorted) name phrases/data/pXXXX.json files whose manifest hashes feed
/// the same derivation the client recomputes from a fetched manifest.
fn phrase_bundle_manifest_guard(cdn_dir: &Path, bundle_json: &[u8]) -> Result<[u8; 32], String> {
    let text = String::from_utf8(bundle_json.to_vec())
        .map_err(|_| "phrase bundle source is not valid UTF-8".to_string())?;
    let bundle: std::collections::BTreeMap<String, serde_json::Value> =
        serde_json::from_str(&text).map_err(|e| format!("failed to parse phrase bundle: {e}"))?;

    let mut hex_hashes = Vec::with_capacity(bundle.len());
    for key in bundle.keys() {
        let chunk = read(cdn_dir, &format!("phrases/data/{key}.json"))?;
        hex_hashes.push(sha256_hex(&chunk));
    }
    let hex_refs: Vec<&str> = hex_hashes.iter().map(String::as_str).collect();
    Ok(cdn_blob::manifest_guard_from_hex_hashes(&hex_refs))
}

fn build_pitch_index_blob(cdn_dir: &Path) -> Result<bool, String> {
    build_single_source_index(cdn_dir, PITCH_INDEX_SOURCE, PITCH_INDEX_BLOB, |text| {
        let index = origa::dictionary::pitch_audio::PitchAudioIndex::from_json(text)
            .map_err(|e| format!("failed to build pitch audio index: {e}"))?;
        serialize_pitch_index_blob_to_rkyv(&index.to_blob())
            .map_err(|e| format!("failed to serialize pitch audio index blob: {e}"))
    })
}

fn default_cdn_dir() -> PathBuf {
    crate::utils::cdn_dir()
}

pub fn run_build_cdn_rkyv(cdn_dir: Option<PathBuf>) -> Result<(), OrigaError> {
    let cdn_dir = cdn_dir.unwrap_or_else(default_cdn_dir);

    let furigana_rebuilt =
        build_furigana(&cdn_dir).map_err(|reason| OrigaError::RepositoryError { reason })?;
    let vocabulary_rebuilt =
        build_vocabulary(&cdn_dir).map_err(|reason| OrigaError::RepositoryError { reason })?;
    let phrase_rebuilt = build_phrase_index_blob(&cdn_dir)
        .map_err(|reason| OrigaError::RepositoryError { reason })?;
    let phrase_v3_rebuilt = build_phrase_index_v3_blob(&cdn_dir)
        .map_err(|reason| OrigaError::RepositoryError { reason })?;
    let bundle_blobs_rebuilt = build_phrase_bundle_blobs(&cdn_dir)
        .map_err(|reason| OrigaError::RepositoryError { reason })?;
    let pitch_rebuilt = build_pitch_index_blob(&cdn_dir)
        .map_err(|reason| OrigaError::RepositoryError { reason })?;

    if !furigana_rebuilt
        && !vocabulary_rebuilt
        && !phrase_rebuilt
        && !phrase_v3_rebuilt
        && !bundle_blobs_rebuilt
        && !pitch_rebuilt
    {
        println!("All CDN rkyv blobs are fresh, nothing to do");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_cdn(contents: &[(String, Vec<u8>)]) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("origa-rkyv-test-{}", ulid::Ulid::new()));
        fs::create_dir_all(dir.join("dictionaries")).unwrap();
        fs::create_dir_all(dir.join("dictionary")).unwrap();
        fs::create_dir_all(dir.join("phrases")).unwrap();
        fs::create_dir_all(dir.join("phrases/data")).unwrap();
        fs::create_dir_all(dir.join("pitch")).unwrap();
        for (path, bytes) in contents {
            fs::write(dir.join(path), bytes).unwrap();
        }
        dir
    }

    fn furigana_source() -> Vec<u8> {
        b"\xe6\x8c\x87|\xe3\x82\x86\xe3\x81\xb3|0:\xe3\x82\x86\xe3\x81\xb3\n".to_vec()
    }

    fn chunk_source() -> Vec<u8> {
        br#"{"\u732b": {"russian_translation": "\u043a\u043e\u0442"}}"#.to_vec()
    }

    fn all_sources() -> Vec<(String, Vec<u8>)> {
        let mut files = vec![
            (FURIGANA_SOURCE.to_string(), furigana_source()),
            (
                PHRASE_INDEX_SOURCE.to_string(),
                br#"{"v":1,"h":"t","total":1,"phrases":[{"i":"01KPJ5S3N1DRFFD236Z4EZ03HJ","t":["hello"],"c":0}]}"#
                    .to_vec(),
            ),
            (
                PITCH_INDEX_SOURCE.to_string(),
                br#"{"v":2,"total":1,"entries":{"\u732b":{"f":"a1b2c3d4.opus","p":1}}}"#.to_vec(),
            ),
        ];
        for index in 1..=CHUNK_COUNT {
            files.push((chunk_path(index), chunk_source()));
        }
        files.extend(bundle_sources());
        files
    }

    fn bundle_sources() -> Vec<(String, Vec<u8>)> {
        let chunk_a = br#"{"i":"01A","t":["hello"],"c":0}"#.to_vec();
        let chunk_b = br#"{"i":"01B","t":["world"],"c":0}"#.to_vec();
        let mut bundle = String::from("{");
        bundle.push_str(r#""p0000":"#);
        bundle.push_str(std::str::from_utf8(&chunk_a).unwrap());
        bundle.push_str(r#","p0001":"#);
        bundle.push_str(std::str::from_utf8(&chunk_b).unwrap());
        bundle.push('}');

        vec![
            ("phrases/data/p0000.json".to_string(), chunk_a),
            ("phrases/data/p0001.json".to_string(), chunk_b),
            (phrase_bundle_source_path(0), bundle.into_bytes()),
        ]
    }

    #[test]
    fn build_writes_blobs_and_second_run_skips() {
        // Arrange
        let dir = temp_cdn(&all_sources());

        // Act
        let first_furigana = build_furigana(&dir).unwrap();
        let first_vocabulary = build_vocabulary(&dir).unwrap();
        let first_phrase = build_phrase_index_blob(&dir).unwrap();
        let first_pitch = build_pitch_index_blob(&dir).unwrap();
        let second_furigana = build_furigana(&dir).unwrap();
        let second_vocabulary = build_vocabulary(&dir).unwrap();
        let second_phrase = build_phrase_index_blob(&dir).unwrap();
        let second_pitch = build_pitch_index_blob(&dir).unwrap();

        // Assert
        assert!(first_furigana, "first run rebuilds");
        assert!(first_vocabulary, "first run rebuilds");
        assert!(first_phrase, "first run rebuilds");
        assert!(first_pitch, "first run rebuilds");
        assert!(!second_furigana, "fresh blob is skipped");
        assert!(!second_vocabulary, "fresh blob is skipped");
        assert!(!second_phrase, "fresh blob is skipped");
        assert!(!second_pitch, "fresh blob is skipped");

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn changed_phrase_source_invalidates_freshness() {
        // Arrange
        let dir = temp_cdn(&all_sources());
        build_phrase_index_blob(&dir).unwrap();

        // Act: edit the source
        fs::write(
            dir.join(PHRASE_INDEX_SOURCE),
            br#"{"v":1,"h":"t2","total":0,"phrases":[]}"#,
        )
        .unwrap();
        let rebuilt = build_phrase_index_blob(&dir).unwrap();

        // Assert
        assert!(rebuilt, "stale blob is regenerated");

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn changed_pitch_source_invalidates_freshness() {
        // Arrange
        let dir = temp_cdn(&all_sources());
        build_pitch_index_blob(&dir).unwrap();

        // Act: edit the source
        fs::write(
            dir.join(PITCH_INDEX_SOURCE),
            br#"{"v":3,"total":0,"entries":{}}"#,
        )
        .unwrap();
        let rebuilt = build_pitch_index_blob(&dir).unwrap();

        // Assert
        assert!(rebuilt, "stale blob is regenerated");

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn phrase_bundle_blob_builds_and_second_run_skips() {
        // Arrange
        let dir = temp_cdn(&all_sources());

        // Act
        let first = build_phrase_bundle_blob(&dir, 0).unwrap();
        let second = build_phrase_bundle_blob(&dir, 0).unwrap();

        // Assert
        assert!(first, "first run rebuilds");
        assert!(!second, "fresh bundle blob is skipped");

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn changed_phrase_bundle_source_invalidates_freshness() {
        // Arrange
        let dir = temp_cdn(&all_sources());
        build_phrase_bundle_blob(&dir, 0).unwrap();

        // Act: edit the bundle source — the new chunk key makes the guard
        // derivation read phrases/data/p0002.json, which does not exist yet.
        let path = dir.join(phrase_bundle_source_path(0));
        let mut json = String::from_utf8(fs::read(&path).unwrap()).unwrap();
        json.pop();
        json.push_str(r#","p0002":[]}"#);
        fs::write(&path, json).unwrap();
        let missing_chunk = build_phrase_bundle_blob(&dir, 0);

        // The chunk file lands on disk (bundle_phrases_data.py always runs
        // against a complete chunk directory) — now the rebuild succeeds.
        fs::write(dir.join("phrases/data/p0002.json"), b"[]").unwrap();
        let rebuilt = build_phrase_bundle_blob(&dir, 0).unwrap();

        // Assert
        assert!(
            missing_chunk.is_err(),
            "missing chunk fails the guard derivation loudly"
        );
        assert!(rebuilt, "stale bundle blob is regenerated");

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn phrase_bundle_guard_binds_to_chunk_hashes() {
        // Arrange
        let dir = temp_cdn(&all_sources());
        build_phrase_bundle_blob(&dir, 0).unwrap();

        // Act: inflate and read the guard back
        let deflated = fs::read(dir.join(phrase_bundle_blob_path(0))).unwrap();
        let inflated = inflate_blob(&deflated).unwrap();
        let (header, payload) = split_blob(&inflated).unwrap();

        // Assert: the guard equals sha256 over the sorted chunk hash hexes
        let expected = cdn_blob::manifest_guard_from_hex_hashes(&[
            &sha256_hex(b"{\"i\":\"01A\",\"t\":[\"hello\"],\"c\":0}"),
            &sha256_hex(b"{\"i\":\"01B\",\"t\":[\"world\"],\"c\":0}"),
        ]);
        assert_eq!(header.manifest_guard, expected);
        assert_eq!(payload, b"{\"p0000\":{\"i\":\"01A\",\"t\":[\"hello\"],\"c\":0},\"p0001\":{\"i\":\"01B\",\"t\":[\"world\"],\"c\":0}}");

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn phrase_and_pitch_blobs_are_deterministic_for_identical_sources() {
        // Arrange
        let first_dir = temp_cdn(&all_sources());
        let second_dir = temp_cdn(&all_sources());

        // Act
        for dir in [&first_dir, &second_dir] {
            build_phrase_index_blob(dir).unwrap();
            build_pitch_index_blob(dir).unwrap();
        }

        // Assert
        assert_eq!(
            fs::read(first_dir.join(PHRASE_INDEX_BLOB)).unwrap(),
            fs::read(second_dir.join(PHRASE_INDEX_BLOB)).unwrap(),
            "identical sources must produce identical phrase blobs"
        );
        assert_eq!(
            fs::read(first_dir.join(PITCH_INDEX_BLOB)).unwrap(),
            fs::read(second_dir.join(PITCH_INDEX_BLOB)).unwrap(),
            "identical sources must produce identical pitch blobs"
        );

        fs::remove_dir_all(&first_dir).ok();
        fs::remove_dir_all(&second_dir).ok();
    }

    #[test]
    fn blobs_are_deterministic_for_identical_sources() {
        // Arrange
        let first_dir = temp_cdn(&all_sources());
        let second_dir = temp_cdn(&all_sources());

        // Act
        build_vocabulary(&first_dir).unwrap();
        build_vocabulary(&second_dir).unwrap();

        // Assert
        let first = fs::read(first_dir.join(VOCABULARY_BLOB)).unwrap();
        let second = fs::read(second_dir.join(VOCABULARY_BLOB)).unwrap();
        assert_eq!(
            first, second,
            "identical sources must produce identical blobs"
        );

        fs::remove_dir_all(&first_dir).ok();
        fs::remove_dir_all(&second_dir).ok();
    }
}
