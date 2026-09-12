//! JSON-chunk parsing: the fallback path that builds a `VocabularyDatabase`
//! from the eleven CDN chunk files (also used by the offline blob builder).

use std::collections::HashMap;

use serde::Deserialize;

use super::blob::VocabularyDatabase;
use super::info::VocabularyInfo;
use crate::domain::OrigaError;

#[derive(Clone, Deserialize)]
pub struct VocabularyChunkData {
    pub chunk_01: String,
    pub chunk_02: String,
    pub chunk_03: String,
    pub chunk_04: String,
    pub chunk_05: String,
    pub chunk_06: String,
    pub chunk_07: String,
    pub chunk_08: String,
    pub chunk_09: String,
    pub chunk_10: String,
    pub chunk_11: String,
}

#[derive(Clone, Deserialize)]
#[serde(untagged)]
enum TranslationValue {
    Structured { t: Vec<String>, d: String },
    Raw(String),
}

impl TranslationValue {
    fn into_translations_and_description(self) -> (Vec<String>, Option<String>) {
        match self {
            TranslationValue::Structured { t, d } => {
                let desc = if d.trim().is_empty() {
                    None
                } else {
                    Some(d.trim().to_string())
                };
                let translations = split_semicolon_joined_translations(t);
                (translations, desc)
            },
            TranslationValue::Raw(s) => {
                tracing::warn!(
                    "Legacy vocabulary format detected, consider migrating to structured format"
                );
                parse_legacy_translation(&s)
            },
        }
    }
}

fn parse_legacy_translation(text: &str) -> (Vec<String>, Option<String>) {
    let mut translations = Vec::new();
    let mut description_parts = Vec::new();

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some(t) = trimmed.strip_prefix("- ") {
            if !t.is_empty() {
                translations.push(t.to_string());
            }
        } else if let Some(d) = trimmed.strip_prefix("> ") {
            if !d.is_empty() {
                description_parts.push(d.to_string());
            }
        } else if !translations.is_empty() {
            description_parts.push(trimmed.to_string());
        } else {
            translations.push(trimmed.to_string());
        }
    }

    if translations.is_empty() && !text.trim().is_empty() {
        translations.push(text.trim().to_string());
    }

    let description = if description_parts.is_empty() {
        None
    } else {
        Some(description_parts.join(" "))
    };
    (translations, description)
}

fn split_semicolon_joined_translations(translations: Vec<String>) -> Vec<String> {
    let mut out = Vec::with_capacity(translations.len());
    for entry in translations {
        let trimmed = entry.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.contains(';') {
            for part in trimmed.split(';') {
                let part = part.trim();
                if !part.is_empty() {
                    out.push(part.to_string());
                }
            }
        } else {
            out.push(trimmed.to_string());
        }
    }
    out
}

#[derive(Deserialize)]
struct VocabularyEntryStoredType {
    russian_translation: Option<String>,
    english_translation: Option<String>,
    ru: Option<TranslationValue>,
    en: Option<TranslationValue>,
    vi: Option<TranslationValue>,
    ko: Option<TranslationValue>,
}

fn resolve_translations(
    entry: &VocabularyEntryStoredType,
    lang: TranslationLang,
) -> (Vec<String>, Option<String>) {
    let (structured, raw) = match lang {
        TranslationLang::Ru => (&entry.ru, &entry.russian_translation),
        TranslationLang::En => (&entry.en, &entry.english_translation),
        // KO/VI chunks are generated in structured form only.
        TranslationLang::Vi => (&entry.vi, &None),
        TranslationLang::Ko => (&entry.ko, &None),
    };

    if let Some(tv) = structured {
        return tv.clone().into_translations_and_description();
    }

    if let Some(raw_str) = raw {
        return parse_legacy_translation(raw_str);
    }

    (vec![], None)
}

enum TranslationLang {
    Ru,
    En,
    Vi,
    Ko,
}

impl VocabularyDatabase {
    pub(super) fn from_chunks(data: VocabularyChunkData) -> Result<Self, OrigaError> {
        fn strip_bom(json: &str) -> &str {
            json.strip_prefix('\u{FEFF}').unwrap_or(json)
        }

        let parse_chunk = |json: &str, chunk_name: &str| {
            serde_json::from_str::<HashMap<String, VocabularyEntryStoredType>>(strip_bom(json))
                .map_err(|e| OrigaError::VocabularyParseError {
                    reason: format!("Failed to parse {}: {}", chunk_name, e),
                })
        };

        let vocabulary_data: HashMap<_, _> = parse_chunk(&data.chunk_01, "chunk_01")?
            .into_iter()
            .chain(parse_chunk(&data.chunk_02, "chunk_02")?)
            .chain(parse_chunk(&data.chunk_03, "chunk_03")?)
            .chain(parse_chunk(&data.chunk_04, "chunk_04")?)
            .chain(parse_chunk(&data.chunk_05, "chunk_05")?)
            .chain(parse_chunk(&data.chunk_06, "chunk_06")?)
            .chain(parse_chunk(&data.chunk_07, "chunk_07")?)
            .chain(parse_chunk(&data.chunk_08, "chunk_08")?)
            .chain(parse_chunk(&data.chunk_09, "chunk_09")?)
            .chain(parse_chunk(&data.chunk_10, "chunk_10")?)
            .chain(parse_chunk(&data.chunk_11, "chunk_11")?)
            .collect();

        let vocabulary_map = vocabulary_data
            .into_iter()
            .map(|(word, entry)| {
                let (ru_translations, ru_description) =
                    resolve_translations(&entry, TranslationLang::Ru);
                let (en_translations, en_description) =
                    resolve_translations(&entry, TranslationLang::En);
                let (vi_translations, vi_description) =
                    resolve_translations(&entry, TranslationLang::Vi);
                let (ko_translations, ko_description) =
                    resolve_translations(&entry, TranslationLang::Ko);

                (
                    word.clone(),
                    VocabularyInfo {
                        word,
                        ru_translations,
                        ru_description,
                        en_translations,
                        en_description,
                        vi_translations,
                        vi_description,
                        ko_translations,
                        ko_description,
                    },
                )
            })
            .collect::<HashMap<String, VocabularyInfo>>();

        Ok(Self { vocabulary_map })
    }
}

/// Build a database from chunk JSONs without installing it into the global
/// slot. Used by the CDN blob builder and by loader tests.
pub fn build_vocabulary_database_from_chunks(
    data: VocabularyChunkData,
) -> Result<VocabularyDatabase, OrigaError> {
    VocabularyDatabase::from_chunks(data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::NativeLanguage;

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

    fn make_valid_chunk_json() -> String {
        r#"{
            "猫": {
                "level": "N5",
                "russian_translation": "- кошка\n- кот",
                "english_translation": "- cat"
            },
            "犬": {
                "level": "N5",
                "russian_translation": "собака",
                "english_translation": "dog"
            }
        }"#
        .to_string()
    }

    fn make_structured_chunk_json() -> String {
        r#"{
            "猫": {
                "level": "N5",
                "ru": { "t": ["кошка", "кот"], "d": "домашнее животное" },
                "en": { "t": ["cat"], "d": "" }
            },
            "犬": {
                "level": "N5",
                "ru": { "t": ["собака"], "d": "" },
                "en": { "t": ["dog"], "d": "domestic animal" }
            }
        }"#
        .to_string()
    }

    fn make_mixed_chunk_json() -> String {
        r#"{
            "猫": {
                "level": "N5",
                "russian_translation": "- кошка\n- кот",
                "english_translation": "- cat"
            },
            "犬": {
                "level": "N5",
                "ru": { "t": ["собака"], "d": "" },
                "en": { "t": ["dog"], "d": "" }
            }
        }"#
        .to_string()
    }

    #[test]
    fn from_chunks_valid_json_loads_entries() {
        let data = empty_chunk_data_with(&make_valid_chunk_json());
        let db = VocabularyDatabase::from_chunks(data).unwrap();
        assert!(db.get_vocabulary_info("猫").is_some());
        assert!(db.get_vocabulary_info("犬").is_some());
        assert!(db.get_vocabulary_info("魚").is_none());
    }

    #[test]
    fn from_chunks_structured_format_loads_entries() {
        let data = empty_chunk_data_with(&make_structured_chunk_json());
        let db = VocabularyDatabase::from_chunks(data).unwrap();
        assert!(db.get_vocabulary_info("猫").is_some());
        assert!(db.get_vocabulary_info("犬").is_some());
    }

    #[test]
    fn korean_vietnamese_translations_use_chunk_fields() {
        let json = r#"{
            "猫": {
                "ru": { "t": ["кошка"], "d": "" },
                "en": { "t": ["cat"], "d": "" },
                "vi": { "t": ["con mèo"], "d": "" },
                "ko": { "t": ["고양이"], "d": "" }
            },
            "犬": {
                "ru": { "t": ["собака"], "d": "" },
                "en": { "t": ["dog"], "d": "" }
            }
        }"#
        .to_string();
        let data = empty_chunk_data_with(&json);
        let db = VocabularyDatabase::from_chunks(data).unwrap();

        let cat = db.get_vocabulary_info("猫").unwrap();
        assert_eq!(
            cat.translations(&NativeLanguage::Korean),
            &["고양이".to_string()]
        );
        assert_eq!(
            cat.translations(&NativeLanguage::Vietnamese),
            &["con mèo".to_string()]
        );
        assert_eq!(cat.korean_translation(), "- 고양이");
        assert_eq!(cat.vietnamese_translation(), "- con mèo");
        assert_eq!(
            db.get_translation("猫", &NativeLanguage::Korean).unwrap(),
            "- 고양이"
        );

        // Legacy entry without vi/ko fields degrades to English.
        let dog = db.get_vocabulary_info("犬").unwrap();
        assert_eq!(
            dog.translations(&NativeLanguage::Korean),
            dog.translations(&NativeLanguage::English)
        );
        assert_eq!(dog.vietnamese_translation(), dog.english_translation());
    }

    #[test]
    fn from_chunks_mixed_formats_work() {
        let data = empty_chunk_data_with(&make_mixed_chunk_json());
        let db = VocabularyDatabase::from_chunks(data).unwrap();

        let cat_ru = db.get_translation("猫", &NativeLanguage::Russian).unwrap();
        assert!(cat_ru.contains("кошка"));

        let dog_ru = db.get_translation("犬", &NativeLanguage::Russian).unwrap();
        assert!(dog_ru.contains("собака"));
    }

    #[test]
    fn from_chunks_strips_bom_prefix() {
        let json_with_bom = format!("\u{FEFF}{}", make_valid_chunk_json());
        let data = empty_chunk_data_with(&json_with_bom);
        let db = VocabularyDatabase::from_chunks(data).unwrap();
        assert!(db.get_vocabulary_info("猫").is_some());
    }

    #[test]
    fn from_chunks_empty_all_chunks_succeeds() {
        let empty = "{}".to_string();
        let data = VocabularyChunkData {
            chunk_01: empty.clone(),
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
        };
        let db = VocabularyDatabase::from_chunks(data).unwrap();
        assert!(db.get_vocabulary_info("anything").is_none());
    }

    #[test]
    fn from_chunks_invalid_json_returns_error() {
        let data = empty_chunk_data_with("not valid json");
        let result = VocabularyDatabase::from_chunks(data);
        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(OrigaError::VocabularyParseError { .. })
        ));
    }

    #[test]
    fn structured_format_splits_semicolon_joined_translations() {
        let data = empty_chunk_data_with(
            r#"{
                "意思": {
                    "level": "N5",
                    "ru": { "t": ["намерение; Воля; Цель", "смысл; Значение; Суть"], "d": "" },
                    "en": { "t": ["intention; Will; Purpose"], "d": "" }
                }
            }"#,
        );
        let db = VocabularyDatabase::from_chunks(data).unwrap();
        let ru = db
            .get_translations("意思", &NativeLanguage::Russian)
            .unwrap();
        assert_eq!(
            ru,
            vec!["намерение", "Воля", "Цель", "смысл", "Значение", "Суть"]
        );
        let en = db
            .get_translations("意思", &NativeLanguage::English)
            .unwrap();
        assert_eq!(en, vec!["intention", "Will", "Purpose"]);
    }

    #[rstest::rstest]
    #[case::dash_prefixed("- кошка\n- кот", vec!["кошка", "кот"], None)]
    #[case::plain_text("собака", vec!["собака"], None)]
    #[case::with_description(
        "- кошка\n> домашнее животное",
        vec!["кошка"],
        Some("домашнее животное")
    )]
    #[case::empty_string("", vec![], None)]
    fn parse_legacy_translation_extracts_translations_and_description(
        #[case] input: &str,
        #[case] expected_translations: Vec<&str>,
        #[case] expected_description: Option<&str>,
    ) {
        let (translations, desc) = parse_legacy_translation(input);
        assert_eq!(
            translations,
            expected_translations
                .into_iter()
                .map(String::from)
                .collect::<Vec<_>>()
        );
        assert_eq!(
            desc,
            expected_description.map(str::to_string),
            "input: {input:?}"
        );
    }

    #[rstest::rstest]
    #[case::clean_input(
        vec!["кошка".to_string(), "кот".to_string()],
        vec!["кошка", "кот"]
    )]
    #[case::empty_parts_dropped(vec!["кошка;;кот;".to_string()], vec!["кошка", "кот"])]
    #[case::empty_entries_dropped(
        vec!["кошка".to_string(), "   ".to_string()],
        vec!["кошка"]
    )]
    fn split_semicolon_joined_translations_normalizes_input(
        #[case] input: Vec<String>,
        #[case] expected: Vec<&str>,
    ) {
        let out = split_semicolon_joined_translations(input);
        assert_eq!(
            out,
            expected.into_iter().map(String::from).collect::<Vec<_>>()
        );
    }
}
