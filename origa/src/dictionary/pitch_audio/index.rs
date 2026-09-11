//! Pitch audio index: the owned JSON-built form, its deterministic CDN
//! blob twin and the zero-copy archived view.

use std::collections::{BTreeMap, HashMap};

use rkyv::util::AlignedVec;
use serde::Deserialize;

use crate::domain::OrigaError;

#[derive(Debug, Clone, PartialEq, Eq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct PitchAudioEntry {
    file: String,
    pitch: Option<u8>,
}

impl PitchAudioEntry {
    pub fn file(&self) -> &str {
        &self.file
    }

    pub fn pitch(&self) -> Option<u8> {
        self.pitch
    }

    pub fn cdn_path(&self) -> String {
        format!("pitch/audio/{}", self.file)
    }
}

pub struct PitchAudioIndex {
    pub(super) entries: HashMap<String, PitchAudioEntry>,
    pub(super) version: u32,
}

#[derive(Deserialize)]
struct IndexFile {
    #[serde(rename = "v")]
    version: u32,
    #[serde(rename = "total")]
    _total: u32,
    entries: HashMap<String, IndexEntryRaw>,
}

#[derive(Deserialize)]
struct IndexEntryRaw {
    #[serde(rename = "f")]
    file: String,
    #[serde(rename = "p")]
    pitch: Option<u8>,
}

impl PitchAudioIndex {
    pub fn from_json(json: &str) -> Result<Self, OrigaError> {
        let file: IndexFile =
            serde_json::from_str(json).map_err(|e| OrigaError::PitchAudioParseError {
                reason: format!("Failed to parse pitch audio index: {}", e),
            })?;

        let entries: HashMap<String, PitchAudioEntry> = file
            .entries
            .into_iter()
            .map(|(word, raw)| {
                (
                    word,
                    PitchAudioEntry {
                        file: raw.file,
                        pitch: raw.pitch,
                    },
                )
            })
            .collect();

        Ok(Self {
            entries,
            version: file.version,
        })
    }

    pub fn get_entry(&self, word: &str) -> Option<&PitchAudioEntry> {
        self.entries.get(word)
    }

    /// Lookup by word+reading with fallback chain: "word|reading" → "reading" → "word".
    pub fn find_audio_for_reading(&self, word: &str, reading: &str) -> Option<&PitchAudioEntry> {
        let composite = format!("{}|{}", word, reading);
        self.get_entry(&composite)
            .or_else(|| self.get_entry(reading))
            .or_else(|| self.get_entry(word))
    }

    pub(super) fn len(&self) -> usize {
        self.entries.len()
    }
}

/// Deterministic CDN-blob form of the index (`BTreeMap` keeps the
/// serialized bytes — and with them the manifest hash — stable across
/// builds; the runtime `HashMap` iterates randomly).
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct PitchAudioIndexBlob {
    pub(super) entries: BTreeMap<String, PitchAudioEntry>,
    pub(super) version: u32,
}

impl PitchAudioIndex {
    /// Deterministic CDN-blob form of this index.
    pub fn to_blob(&self) -> PitchAudioIndexBlob {
        PitchAudioIndexBlob {
            entries: self.entries.clone().into_iter().collect(),
            version: self.version,
        }
    }
}

/// Serialize the deterministic blob form (payload of a CDN blob).
pub fn serialize_pitch_index_blob_to_rkyv(
    blob: &PitchAudioIndexBlob,
) -> Result<Vec<u8>, OrigaError> {
    rkyv::to_bytes::<rkyv::rancor::Error>(blob)
        .map(|bytes| bytes.to_vec())
        .map_err(|e| OrigaError::PitchAudioParseError {
            reason: format!("failed to serialize pitch audio index blob: {e}"),
        })
}

impl ArchivedPitchAudioIndexBlob {
    pub fn get_entry(&self, word: &str) -> Option<PitchAudioEntry> {
        self.entries.get(word).map(clone_entry)
    }

    /// Lookup by word+reading with fallback chain:
    /// "word|reading" → "reading" → "word".
    pub fn find_audio_for_reading(&self, word: &str, reading: &str) -> Option<PitchAudioEntry> {
        let composite = format!("{}|{}", word, reading);
        self.get_entry(&composite)
            .or_else(|| self.get_entry(reading))
            .or_else(|| self.get_entry(word))
    }

    pub fn version(&self) -> u32 {
        u32::from(self.version)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

fn clone_entry(entry: &rkyv::Archived<PitchAudioEntry>) -> PitchAudioEntry {
    PitchAudioEntry {
        file: entry.file.as_str().to_string(),
        pitch: entry.pitch.as_ref().copied(),
    }
}

/// Zero-copy archived view of a blob payload. The payload is copied once
/// into an immortal `AlignedVec` (checked `access` requires properly
/// aligned bytes); a validation failure leaves the leaked bytes
/// unreclaimed until reload while callers fall back to the JSON path.
pub fn access_pitch_index_blob(
    payload: &[u8],
) -> Result<&'static ArchivedPitchAudioIndexBlob, OrigaError> {
    let aligned: &'static AlignedVec = Box::leak(Box::new({
        let mut buffer = AlignedVec::new();
        buffer.extend_from_slice(payload);
        buffer
    }));
    rkyv::access::<ArchivedPitchAudioIndexBlob, rkyv::rancor::Error>(aligned.as_slice()).map_err(
        |e| OrigaError::PitchAudioParseError {
            reason: format!("failed to access pitch audio index blob: {e}"),
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn index_json() -> &'static str {
        r#"{"v":2,"total":3,"entries":{"猫":{"f":"a1b2c3d4.opus","p":1},"食べる":{"f":"e5f6a7b8.opus","p":0},"あ":{"f":"c9d0e1f2.opus","p":null}}}"#
    }

    fn index_v3_json() -> &'static str {
        r#"{"v":3,"total":5,"entries":{"役|やく":{"f":"yaku.opus","p":1},"役|えき":{"f":"eki.opus","p":0},"やく":{"f":"yaku_kana.opus","p":1},"えき":{"f":"eki_kana.opus","p":0},"役":{"f":"fallback.opus","p":0}}}"#
    }

    fn built_index() -> PitchAudioIndex {
        PitchAudioIndex::from_json(index_json()).expect("valid JSON")
    }

    fn built_view() -> &'static ArchivedPitchAudioIndexBlob {
        let index = built_index();
        let payload = serialize_pitch_index_blob_to_rkyv(&index.to_blob()).unwrap();
        access_pitch_index_blob(&payload).unwrap()
    }

    #[test]
    fn from_json_valid() {
        let index = built_index();
        assert_eq!(index.version, 2);
        assert_eq!(index.entries.len(), 3);
    }

    #[test]
    fn from_json_invalid() {
        let result = PitchAudioIndex::from_json("not json");
        assert!(result.is_err());
    }

    #[test]
    fn get_entry_found() {
        let index = built_index();
        let entry = index.get_entry("猫").expect("entry should exist");
        assert_eq!(entry.file(), "a1b2c3d4.opus");
        assert_eq!(entry.pitch(), Some(1));
    }

    #[test]
    fn get_entry_not_found() {
        let index = built_index();
        assert!(index.get_entry("犬").is_none());
    }

    #[test]
    fn entry_with_null_pitch() {
        let index = built_index();
        let entry = index.get_entry("あ").expect("entry should exist");
        assert_eq!(entry.pitch(), None);
    }

    #[test]
    fn find_audio_for_reading_prefers_composite_then_reading_then_word() {
        let index = PitchAudioIndex::from_json(index_v3_json()).expect("valid JSON");

        let entry = index
            .find_audio_for_reading("役", "やく")
            .expect("composite should match");
        assert_eq!(entry.file(), "yaku.opus");

        let entry = index
            .find_audio_for_reading("NotExist", "えき")
            .expect("kana fallback");
        assert_eq!(entry.file(), "eki_kana.opus");

        let entry = index
            .find_audio_for_reading("役", "NotExist")
            .expect("kanji fallback");
        assert_eq!(entry.file(), "fallback.opus");
    }

    #[test]
    fn get_entry_covers_composite_kana_and_kanji_keys() {
        let index = PitchAudioIndex::from_json(index_v3_json()).expect("valid JSON");
        assert_eq!(index.get_entry("役|やく").unwrap().file(), "yaku.opus");
        assert_eq!(index.get_entry("えき").unwrap().file(), "eki_kana.opus");
        assert_eq!(index.get_entry("役").unwrap().file(), "fallback.opus");
        assert!(index.get_entry("NotExist|xyz").is_none());
    }

    #[test]
    fn cdn_path_prefixes_audio_dir() {
        let index = built_index();
        let entry = index.get_entry("猫").unwrap();
        assert_eq!(entry.cdn_path(), "pitch/audio/a1b2c3d4.opus");
    }

    #[test]
    fn blob_serialization_is_deterministic_across_builds() {
        let first = built_index().to_blob();
        let second = built_index().to_blob();
        assert_eq!(
            serialize_pitch_index_blob_to_rkyv(&first).unwrap(),
            serialize_pitch_index_blob_to_rkyv(&second).unwrap()
        );
    }

    /// The archived view must answer exactly like the owned index built
    /// from the same JSON.
    #[rstest::rstest]
    #[case::simple_word("猫", "まる", "a1b2c3d4.opus")]
    #[case::unknown_word("犬", "いぬ", "")]
    fn archived_lookups_match_owned_index(
        #[case] word: &str,
        #[case] reading: &str,
        #[case] expected_file: &str,
    ) {
        let index = built_index();
        let view = built_view();

        let owned = index.find_audio_for_reading(word, reading).cloned();
        let archived = view.find_audio_for_reading(word, reading);
        assert_eq!(archived, owned);

        if expected_file.is_empty() {
            assert!(archived.is_none());
        } else {
            assert_eq!(archived.unwrap().file(), expected_file);
        }
    }

    #[test]
    fn archived_metadata_matches_owned_index() {
        let index = built_index();
        let view = built_view();
        assert_eq!(view.version(), index.version);
        assert_eq!(view.len(), index.len());
    }

    #[test]
    fn access_rejects_truncated_payload() {
        // Arrange: a large enough index that half the payload necessarily
        // cuts structural nodes (tiny archives can survive truncation).
        let mut index = PitchAudioIndex {
            entries: std::collections::HashMap::new(),
            version: 1,
        };
        for i in 0..500 {
            index.entries.insert(
                format!("word{i:04}"),
                PitchAudioEntry {
                    file: format!("f{i:04}.opus"),
                    pitch: Some((i % 5) as u8),
                },
            );
        }
        let payload = serialize_pitch_index_blob_to_rkyv(&index.to_blob()).unwrap();

        // Act
        let result = access_pitch_index_blob(&payload[..payload.len() / 2]);

        // Assert
        assert!(result.is_err());
    }
}
