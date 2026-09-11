//! Phrase detail cache: per-chunk phrase texts and translations, loaded
//! lazily on top of the index.

use std::collections::{HashMap, HashSet};
use std::sync::RwLock;

use serde::Deserialize;
use ulid::Ulid;

use crate::domain::OrigaError;
use crate::domain::value_objects::NativeLanguage;

pub(super) static PHRASE_DATA: RwLock<Option<PhraseDataCache>> = RwLock::new(None);

#[derive(Clone)]
pub struct PhraseDetail {
    pub id: Ulid,
    pub text: String,
    pub translation_ru: Option<String>,
    pub translation_en: Option<String>,
    pub translation_ko: Option<String>,
    pub translation_vi: Option<String>,
}

impl PhraseDetail {
    pub fn translation(&self, lang: &NativeLanguage) -> Option<&str> {
        match lang {
            NativeLanguage::Russian => self.translation_ru.as_deref(),
            // Legacy CDN data without vi/ko fields falls back to English.
            NativeLanguage::Korean => self
                .translation_ko
                .as_deref()
                .or(self.translation_en.as_deref()),
            NativeLanguage::Vietnamese => self
                .translation_vi
                .as_deref()
                .or(self.translation_en.as_deref()),
            NativeLanguage::English => self.translation_en.as_deref(),
        }
    }
}

pub(super) struct PhraseDataCache {
    details: HashMap<Ulid, PhraseDetail>,
    loaded_chunks: HashSet<u32>,
}

impl PhraseDataCache {
    fn new() -> Self {
        Self {
            details: HashMap::new(),
            loaded_chunks: HashSet::new(),
        }
    }

    fn get_detail(&self, id: &Ulid) -> Option<&PhraseDetail> {
        self.details.get(id)
    }

    fn insert_chunk(&mut self, chunk_id: u32, details: Vec<PhraseDetail>) {
        self.loaded_chunks.insert(chunk_id);
        for detail in details {
            self.details.insert(detail.id, detail);
        }
    }

    fn is_chunk_loaded(&self, chunk_id: u32) -> bool {
        self.loaded_chunks.contains(&chunk_id)
    }
}

#[derive(Deserialize)]
struct DetailRaw {
    #[serde(rename = "i")]
    id: Ulid,
    #[serde(rename = "x")]
    text: String,
    #[serde(rename = "ru")]
    translation_ru: Option<String>,
    #[serde(rename = "en")]
    translation_en: Option<String>,
    #[serde(default)]
    #[serde(rename = "ko")]
    translation_ko: Option<String>,
    #[serde(default)]
    #[serde(rename = "vi")]
    translation_vi: Option<String>,
}

const STRAIGHT_DOUBLE_QUOTE: char = '"';
const STRAIGHT_SINGLE_QUOTE: char = '\'';

/// Strip a single outer pair of wrapping quote characters from a translation
/// string, leaving any inner quotes intact.
///
/// Context (#178 P-5): ~12k phrase translations across the dataset are wrapped
/// in an outer pair of straight `"..."` (or, far less often, straight `'...'`).
/// These wrappers are an extraction artifact — the source MT pipeline quoted
/// every translation. Wrappers obscure matches and break card rendering, so we
/// normalize at the load boundary (here) instead of mutating the underlying
/// CDN chunks (which remain byte-stable for hash/version purposes).
///
/// Curly quotes, guillemets (`«...»`), and Japanese corner brackets are
/// intentionally NOT stripped — they are legitimate inline quoting conventions
/// that may appear in dialogue-heavy content.
fn normalize_translation(raw: &str) -> String {
    let trimmed = raw.trim();
    let mut chars = trimmed.chars();
    let Some(first) = chars.next() else {
        return trimmed.to_string();
    };
    let last = trimmed.chars().last();

    let is_straight_pair = matches!(first, STRAIGHT_DOUBLE_QUOTE | STRAIGHT_SINGLE_QUOTE)
        && last == Some(first)
        && trimmed.len() > first.len_utf8();

    if !is_straight_pair {
        return trimmed.to_string();
    }

    let last_len = last.map(|c| c.len_utf8()).unwrap_or(0);
    let inner_end = trimmed.len().saturating_sub(last_len);
    let inner_start = first.len_utf8();
    if inner_end < inner_start {
        return trimmed.to_string();
    }
    trimmed[inner_start..inner_end].to_string()
}

pub fn cache_phrase_details(chunk_id: u32, json: &str) -> Result<(), OrigaError> {
    let raw_list: Vec<DetailRaw> =
        serde_json::from_str(json).map_err(|e| OrigaError::PhraseParseError {
            reason: format!("Failed to parse phrase chunk: {}", e),
        })?;

    let details: Vec<PhraseDetail> = raw_list
        .into_iter()
        .map(|r| PhraseDetail {
            id: r.id,
            text: r.text,
            translation_ru: r.translation_ru.map(|s| normalize_translation(&s)),
            translation_en: r.translation_en.map(|s| normalize_translation(&s)),
            translation_ko: r.translation_ko.map(|s| normalize_translation(&s)),
            translation_vi: r.translation_vi.map(|s| normalize_translation(&s)),
        })
        .collect();

    let mut guard = PHRASE_DATA.write().unwrap_or_else(|e| e.into_inner());
    let cache = guard.get_or_insert_with(PhraseDataCache::new);
    cache.insert_chunk(chunk_id, details);
    Ok(())
}

pub fn get_cached_phrase_detail(id: &Ulid) -> Option<PhraseDetail> {
    let guard = PHRASE_DATA.read().unwrap_or_else(|e| e.into_inner());
    guard.as_ref()?.get_detail(id).cloned()
}

pub fn get_phrase_text(id: &Ulid) -> Option<String> {
    let guard = PHRASE_DATA.read().unwrap_or_else(|e| e.into_inner());
    guard.as_ref()?.get_detail(id).map(|d| d.text.clone())
}

pub fn get_phrase_translation(id: &Ulid, lang: &NativeLanguage) -> Option<String> {
    let guard = PHRASE_DATA.read().unwrap_or_else(|e| e.into_inner());
    guard.as_ref()?.get_detail(id).and_then(|d| match lang {
        NativeLanguage::Russian => d.translation_ru.clone(),
        NativeLanguage::English => d.translation_en.clone(),
        // Legacy CDN data without vi/ko fields falls back to English.
        NativeLanguage::Korean => d.translation_ko.clone().or(d.translation_en.clone()),
        NativeLanguage::Vietnamese => d.translation_vi.clone().or(d.translation_en.clone()),
    })
}

pub fn is_chunk_loaded(chunk_id: u32) -> bool {
    PHRASE_DATA
        .read()
        .unwrap_or_else(|e| e.into_inner())
        .as_ref()
        .map(|cache| cache.is_chunk_loaded(chunk_id))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chunk_json() -> &'static str {
        r#"[{"i":"01KPJ5S3N1DRFFD236Z4EZ03HJ","x":"Hello world","ru":"Привет мир","en":"Hello world"},{"i":"01KPJ5S3N1DRFFD236Z4EZ03HK","x":"Goodbye world","ru":"Прощай мир","en":"Goodbye world"}]"#
    }

    fn first_id() -> Ulid {
        Ulid::from_string("01KPJ5S3N1DRFFD236Z4EZ03HJ").expect("valid ULID")
    }

    #[test]
    fn insert_chunk_then_lookup_detail() {
        let mut cache = PhraseDataCache::new();

        assert!(!cache.is_chunk_loaded(0));
        assert!(cache.get_detail(&first_id()).is_none());

        let details = vec![PhraseDetail {
            id: first_id(),
            text: "Hello world".to_string(),
            translation_ru: Some("Привет мир".to_string()),
            translation_en: Some("Hello world".to_string()),
            translation_ko: None,
            translation_vi: None,
        }];
        cache.insert_chunk(0, details);

        assert!(cache.is_chunk_loaded(0));
        let detail = cache.get_detail(&first_id()).expect("detail should exist");
        assert_eq!(detail.text, "Hello world");
    }

    #[test]
    fn parse_chunk_json() {
        let raw_list: Vec<DetailRaw> =
            serde_json::from_str(chunk_json()).expect("valid chunk JSON");
        assert_eq!(raw_list.len(), 2);
        assert_eq!(raw_list[0].text, "Hello world");
        assert_eq!(raw_list[0].translation_ru, Some("Привет мир".to_string()));
        assert!(raw_list[0].translation_en.is_some());
    }

    #[test]
    fn korean_vietnamese_translations_parse_and_fall_back() {
        let json = r#"[{"i":"01KPJ5S3N1DRFFD236Z4EZ03HJ","x":"こんにちは","ru":"Привет","en":"Hello","ko":"안녕","vi":"Xin chào"},{"i":"01KPJ5S3N1DRFFD236Z4EZ03HK","x":"さようなら","ru":"Прощай","en":"Goodbye"}]"#;
        let raw_list: Vec<DetailRaw> = serde_json::from_str(json).expect("valid chunk JSON");

        assert_eq!(raw_list[0].translation_ko, Some("안녕".to_string()));
        assert_eq!(raw_list[0].translation_vi, Some("Xin chào".to_string()));

        let with_ko = PhraseDetail {
            id: raw_list[0].id,
            text: raw_list[0].text.clone(),
            translation_ru: raw_list[0].translation_ru.clone(),
            translation_en: raw_list[0].translation_en.clone(),
            translation_ko: raw_list[0].translation_ko.clone(),
            translation_vi: raw_list[0].translation_vi.clone(),
        };
        assert_eq!(with_ko.translation(&NativeLanguage::Korean), Some("안녕"));
        assert_eq!(
            with_ko.translation(&NativeLanguage::Vietnamese),
            Some("Xin chào")
        );

        // Legacy entry without ko/vi fields degrades to English.
        let legacy = PhraseDetail {
            id: raw_list[1].id,
            text: raw_list[1].text.clone(),
            translation_ru: raw_list[1].translation_ru.clone(),
            translation_en: raw_list[1].translation_en.clone(),
            translation_ko: None,
            translation_vi: None,
        };
        assert_eq!(
            legacy.translation(&NativeLanguage::Korean),
            legacy.translation(&NativeLanguage::English)
        );
        assert_eq!(
            legacy.translation(&NativeLanguage::Vietnamese),
            Some("Goodbye")
        );
    }

    #[rstest::rstest]
    #[case::straight_double("\"нечто\"", "нечто")]
    #[case::straight_single("'нечто'", "нечто")]
    #[case::inner_preserved("\"нечто\" и \"ещё\"", "нечто\" и \"ещё")]
    #[case::guillemets_kept("«диалог»", "«диалог»")]
    #[case::curly_kept("\u{201C}диалог\u{201D}", "\u{201C}диалог\u{201D}")]
    #[case::unquoted_passthrough("привет мир", "привет мир")]
    #[case::trims_whitespace("  \"нечто\"  ", "нечто")]
    #[case::too_short("\"", "\"")]
    #[case::empty_pair("''", "")]
    #[case::empty_input("", "")]
    #[case::unbalanced_left_alone("\"нечто", "\"нечто")]
    #[case::unbalanced_right_alone("нечто\"", "нечто\"")]
    fn normalize_translation_variants(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(normalize_translation(input), expected);
    }
}
