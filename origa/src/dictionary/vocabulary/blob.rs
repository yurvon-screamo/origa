//! CDN blob forms and zero-copy access. `VocabularyBlob` is the
//! deterministic BTreeMap serialization form deployed to the CDN and
//! cached locally; `access_vocabulary_blob` returns an archived view over
//! an immortal aligned copy of the payload without building owned
//! structures.

use std::collections::{BTreeMap, HashMap};

use rkyv::string::ArchivedString;
use rkyv::util::AlignedVec;
use rkyv::vec::ArchivedVec;

use super::info::VocabularyInfo;
use crate::domain::{NativeLanguage, OrigaError};

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct VocabularyDatabase {
    pub(super) vocabulary_map: HashMap<String, VocabularyInfo>,
}

/// Deterministic serialization form of [`VocabularyDatabase`] for CDN blobs.
///
/// The inner `HashMap` iterates in a per-process random order, so serializing
/// the database directly would produce a byte-different blob on every build
/// run and invalidate every client cache after each deploy. `BTreeMap`
/// serialization is ordered, keeping the blob hash stable for unchanged
/// sources.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct VocabularyBlob {
    pub(super) entries: BTreeMap<String, VocabularyInfo>,
}

impl VocabularyDatabase {
    /// Deterministic CDN-blob form of this database.
    pub fn to_cdn_blob(&self) -> VocabularyBlob {
        VocabularyBlob {
            entries: self.vocabulary_map.clone().into_iter().collect(),
        }
    }
}

impl VocabularyBlob {
    pub fn into_database(self) -> VocabularyDatabase {
        VocabularyDatabase {
            vocabulary_map: self.entries.into_iter().collect(),
        }
    }
}

impl VocabularyDatabase {
    pub fn get_translation(&self, word: &str, native_language: &NativeLanguage) -> Option<String> {
        self.vocabulary_map
            .get(word)
            .map(|info| match native_language {
                NativeLanguage::Russian => info.russian_translation(),
                NativeLanguage::English => info.english_translation(),
                NativeLanguage::Korean => info.korean_translation(),
                NativeLanguage::Vietnamese => info.vietnamese_translation(),
            })
    }

    pub fn get_translations(
        &self,
        word: &str,
        native_language: &NativeLanguage,
    ) -> Option<Vec<String>> {
        self.vocabulary_map
            .get(word)
            .map(|info| info.translations(native_language).to_vec())
    }

    pub fn get_description(&self, word: &str, native_language: &NativeLanguage) -> Option<String> {
        self.vocabulary_map
            .get(word)
            .and_then(|info| info.description(native_language).map(|s| s.to_string()))
    }

    pub fn get_vocabulary_info(&self, word: &str) -> Option<&VocabularyInfo> {
        self.vocabulary_map.get(word)
    }
}

/// Serialize a CDN-blob form of an already-built database (deterministic).
pub fn serialize_vocabulary_blob_to_rkyv(db: &VocabularyDatabase) -> Result<Vec<u8>, OrigaError> {
    rkyv::to_bytes::<rkyv::rancor::Error>(&db.to_cdn_blob())
        .map(|bytes| bytes.to_vec())
        .map_err(|e| OrigaError::VocabularyParseError {
            reason: format!("Failed to serialize vocabulary blob: {}", e),
        })
}

/// Deserialize a CDN-blob payload into a database (owned form, no JSON).
pub fn vocabulary_database_from_blob_rkyv(
    payload: &[u8],
) -> Result<VocabularyDatabase, OrigaError> {
    let blob: VocabularyBlob = rkyv::from_bytes::<VocabularyBlob, rkyv::rancor::Error>(payload)
        .map_err(|e| OrigaError::VocabularyParseError {
            reason: format!("Failed to deserialize vocabulary blob: {}", e),
        })?;
    Ok(blob.into_database())
}

/// Zero-copy archived view of a blob payload.
///
/// The payload is copied once into an immortal `AlignedVec` (rkyv's checked
/// `access` requires properly aligned bytes; `Vec<u8>` only guarantees
/// alignment 1 and a CDN blob header offset would leave the payload at an
/// arbitrary alignment). The leaked buffer lives for the rest of the
/// process — the dictionary is load-once anyway. A validation failure
/// leaves the leaked bytes unreclaimed until reload; callers fall back to
/// the JSON-chunk path, so the app keeps working.
pub fn access_vocabulary_blob(
    payload: &[u8],
) -> Result<&'static ArchivedVocabularyBlob, OrigaError> {
    let aligned: &'static AlignedVec = Box::leak(Box::new({
        let mut buffer = AlignedVec::new();
        buffer.extend_from_slice(payload);
        buffer
    }));
    rkyv::access::<ArchivedVocabularyBlob, rkyv::rancor::Error>(aligned.as_slice()).map_err(|e| {
        OrigaError::VocabularyParseError {
            reason: format!("failed to access vocabulary blob: {e}"),
        }
    })
}

impl ArchivedVocabularyBlob {
    pub fn translation(&self, word: &str, native_language: &NativeLanguage) -> Option<String> {
        let info = self.entries.get(word)?;
        Some(match native_language {
            NativeLanguage::Russian => bullets(&info.ru_translations),
            NativeLanguage::English => bullets(&info.en_translations),
            // KO/VI fall back to English for entries without ko/vi fields.
            NativeLanguage::Korean => pick_bullets(&info.ko_translations, &info.en_translations),
            NativeLanguage::Vietnamese => {
                pick_bullets(&info.vi_translations, &info.en_translations)
            },
        })
    }

    pub fn translations(
        &self,
        word: &str,
        native_language: &NativeLanguage,
    ) -> Option<Vec<String>> {
        let info = self.entries.get(word)?;
        Some(match native_language {
            NativeLanguage::Russian => strings(&info.ru_translations),
            NativeLanguage::English => strings(&info.en_translations),
            NativeLanguage::Korean => pick_strings(&info.ko_translations, &info.en_translations),
            NativeLanguage::Vietnamese => {
                pick_strings(&info.vi_translations, &info.en_translations)
            },
        })
    }

    pub fn description(&self, word: &str, native_language: &NativeLanguage) -> Option<String> {
        let info = self.entries.get(word)?;
        match native_language {
            NativeLanguage::Russian => optional_string(&info.ru_description),
            NativeLanguage::English => optional_string(&info.en_description),
            NativeLanguage::Korean => pick_description(&info.ko_description, &info.en_description),
            NativeLanguage::Vietnamese => {
                pick_description(&info.vi_description, &info.en_description)
            },
        }
    }
}

fn bullets(list: &ArchivedVec<ArchivedString>) -> String {
    list.iter()
        .map(|t| format!("- {}", t.as_str()))
        .collect::<Vec<_>>()
        .join("\n")
}

fn strings(list: &ArchivedVec<ArchivedString>) -> Vec<String> {
    list.iter().map(|t| t.as_str().to_string()).collect()
}

fn optional_string(value: &rkyv::option::ArchivedOption<ArchivedString>) -> Option<String> {
    value.as_ref().map(|s| s.as_str().to_string())
}

fn pick_bullets(
    primary: &ArchivedVec<ArchivedString>,
    fallback: &ArchivedVec<ArchivedString>,
) -> String {
    if primary.is_empty() {
        bullets(fallback)
    } else {
        bullets(primary)
    }
}

fn pick_strings(
    primary: &ArchivedVec<ArchivedString>,
    fallback: &ArchivedVec<ArchivedString>,
) -> Vec<String> {
    if primary.is_empty() {
        strings(fallback)
    } else {
        strings(primary)
    }
}

fn pick_description(
    primary: &rkyv::option::ArchivedOption<ArchivedString>,
    fallback: &rkyv::option::ArchivedOption<ArchivedString>,
) -> Option<String> {
    optional_string(primary).or_else(|| optional_string(fallback))
}

#[cfg(test)]
mod tests {
    use super::super::chunks::VocabularyChunkData;
    use super::super::chunks::build_vocabulary_database_from_chunks;
    use super::*;

    fn empty_chunk_data_with(chunk_01: &str) -> VocabularyChunkData {
        let empty = "{}".to_string();
        VocabularyChunkData {
            chunk_01: chunk_01.to_string(),
            chunk_02: empty.clone(),
            chunk_03: empty.clone(),
            chunk_04: empty.clone(),
            chunk_05: empty.clone(),
            chunk_06: empty.clone(),
            chunk_07: empty.clone(),
            chunk_08: empty.clone(),
            chunk_09: empty.clone(),
            chunk_10: empty.clone(),
            chunk_11: empty,
        }
    }

    fn sample_chunks() -> VocabularyChunkData {
        empty_chunk_data_with(
            r#"{
                "猫": {
                    "ru": { "t": ["кошка", "кот"], "d": "домашнее животное" },
                    "en": { "t": ["cat"], "d": "" },
                    "vi": { "t": ["con mèo"], "d": "" },
                    "ko": { "t": ["고양이"], "d": "" }
                },
                "犬": {
                    "russian_translation": "- собака",
                    "english_translation": "- dog"
                }
            }"#,
        )
    }

    #[test]
    fn cdn_blob_round_trip_preserves_translations() {
        // Arrange
        let db = build_vocabulary_database_from_chunks(sample_chunks()).unwrap();

        // Act
        let payload = serialize_vocabulary_blob_to_rkyv(&db).unwrap();
        let restored = vocabulary_database_from_blob_rkyv(&payload).unwrap();

        // Assert
        let ru = restored
            .get_translations("猫", &NativeLanguage::Russian)
            .unwrap();
        assert_eq!(ru, vec!["кошка", "кот"]);
        let en = restored
            .get_translations("猫", &NativeLanguage::English)
            .unwrap();
        assert_eq!(en, vec!["cat"]);
    }

    #[test]
    fn cdn_blob_serialization_is_deterministic_across_builds() {
        // Arrange: two independently built databases from the same chunks
        let first = build_vocabulary_database_from_chunks(sample_chunks()).unwrap();
        let second = build_vocabulary_database_from_chunks(sample_chunks()).unwrap();

        // Act
        let first_payload = serialize_vocabulary_blob_to_rkyv(&first).unwrap();
        let second_payload = serialize_vocabulary_blob_to_rkyv(&second).unwrap();

        // Assert: byte-identical blobs keep the manifest hash stable across
        // deploys with unchanged sources (no mass client re-download).
        assert_eq!(first_payload, second_payload);
    }

    #[test]
    fn access_rejects_truncated_payload() {
        // Arrange: a checked access must refuse an archive that claims
        // structures beyond the buffer — the dangling-read safety property.
        // (Byte-level content corruption is NOT access's job: it is covered
        // by the CDN blob header guard, see `dictionary::cdn_blob`.)
        let db = build_vocabulary_database_from_chunks(sample_chunks()).unwrap();
        let payload = serialize_vocabulary_blob_to_rkyv(&db).unwrap();
        let truncated = &payload[..payload.len() / 2];

        // Act
        let result = access_vocabulary_blob(truncated);

        // Assert
        assert!(result.is_err());
    }

    /// The archived view must answer exactly like the owned database built
    /// from the same chunks — the zero-copy fast path may not change lookup
    /// semantics, including the KO/VI → EN fallback for legacy entries.
    #[rstest::rstest]
    #[case::structured_ru("猫", NativeLanguage::Russian)]
    #[case::structured_en("猫", NativeLanguage::English)]
    #[case::structured_ko("猫", NativeLanguage::Korean)]
    #[case::structured_vi("猫", NativeLanguage::Vietnamese)]
    #[case::legacy_ru("犬", NativeLanguage::Russian)]
    #[case::legacy_ko_falls_back_to_en("犬", NativeLanguage::Korean)]
    #[case::missing_word("魚", NativeLanguage::Russian)]
    fn archived_blob_lookups_match_owned_database(
        #[case] word: &str,
        #[case] lang: NativeLanguage,
    ) {
        // Arrange
        let db = build_vocabulary_database_from_chunks(sample_chunks()).unwrap();
        let payload = serialize_vocabulary_blob_to_rkyv(&db).unwrap();
        let view = access_vocabulary_blob(&payload).unwrap();

        // Act / Assert
        assert_eq!(
            view.translation(word, &lang),
            db.get_translation(word, &lang),
            "translation mismatch for {word}/{lang:?}"
        );
        assert_eq!(
            view.translations(word, &lang),
            db.get_translations(word, &lang),
            "translations mismatch for {word}/{lang:?}"
        );
        assert_eq!(
            view.description(word, &lang),
            db.get_description(word, &lang),
            "description mismatch for {word}/{lang:?}"
        );
    }
}
