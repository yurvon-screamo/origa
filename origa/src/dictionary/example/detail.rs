//! Example detail cache: per-chunk sentence texts, translations and ruby,
//! loaded lazily on top of the index (mirrors `phrase::detail`).

use std::collections::{HashMap, HashSet};
use std::sync::RwLock;

use serde::Deserialize;

use crate::domain::OrigaError;

/// Sentences per data chunk; must match `CHUNK_SIZE` in
/// `scripts/build_examples.py`.
pub const CHUNK_SIZE: u32 = 1000;

static EXAMPLE_DATA: RwLock<Option<ExampleDataCache>> = RwLock::new(None);

/// Furigana pair: base form + its kana reading.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RubyPair {
    pub surface: String,
    pub reading: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ExampleDetail {
    pub sentence_id: u32,
    pub text: String,
    pub translation_en: Option<String>,
    pub translation_ru: Option<String>,
    pub translation_vi: Option<String>,
    pub translation_ko: Option<String>,
    pub ruby: Vec<RubyPair>,
}

impl ExampleDetail {
    /// Translation for the user's native language with English fallback.
    /// Unlike `PhraseDetail` (where ru/ko/vi are always present), example
    /// translations may lag behind (pipeline fills locales progressively),
    /// so every locale falls back to English rather than to nothing.
    pub fn translation(&self, lang: &crate::domain::value_objects::NativeLanguage) -> Option<&str> {
        use crate::domain::value_objects::NativeLanguage;
        match lang {
            NativeLanguage::Russian => self
                .translation_ru
                .as_deref()
                .or(self.translation_en.as_deref()),
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

#[derive(Deserialize)]
struct DetailRaw {
    #[serde(rename = "i")]
    sentence_id: u32,
    #[serde(rename = "x")]
    text: String,
    #[serde(default)]
    en: Option<String>,
    #[serde(default)]
    ru: Option<String>,
    #[serde(default)]
    vi: Option<String>,
    #[serde(default)]
    ko: Option<String>,
    #[serde(rename = "f", default)]
    ruby: Vec<(String, String)>,
}

struct ExampleDataCache {
    details: HashMap<u32, ExampleDetail>,
    loaded_chunks: HashSet<u32>,
}

fn cache() -> &'static RwLock<Option<ExampleDataCache>> {
    &EXAMPLE_DATA
}

fn with_cache<T>(f: impl FnOnce(&mut ExampleDataCache) -> T) -> Result<T, OrigaError> {
    let mut guard = cache().write().map_err(|_| OrigaError::ExampleParseError {
        reason: "example data lock poisoned".to_string(),
    })?;
    let Some(c) = guard.as_mut() else {
        // Entry is created below before use; a poisoned-then-recovered lock
        // could theoretically expose None — degrade to an empty cache.
        *guard = Some(ExampleDataCache::new());
        let Some(c) = guard.as_mut() else {
            return Err(OrigaError::ExampleParseError {
                reason: "example data cache unavailable".to_string(),
            });
        };
        return Ok(f(c));
    };
    Ok(f(c))
}

impl ExampleDataCache {
    fn new() -> Self {
        Self {
            details: HashMap::new(),
            loaded_chunks: HashSet::new(),
        }
    }
}

/// Install one chunk of raw `cdn/examples/data/sNNNN.json` text.
/// Re-installing a chunk is a no-op.
pub fn cache_example_details(chunk_json: &str) -> Result<(), OrigaError> {
    let raw: Vec<DetailRaw> =
        serde_json::from_str(chunk_json).map_err(|e| OrigaError::ExampleParseError {
            reason: format!("example chunk: {e}"),
        })?;
    with_cache(|c| {
        if let Some(first) = raw.first() {
            let chunk = first.sentence_id / CHUNK_SIZE;
            if c.loaded_chunks.contains(&chunk) {
                return;
            }
            c.loaded_chunks.insert(chunk);
            for d in raw {
                c.details.insert(
                    d.sentence_id,
                    ExampleDetail {
                        sentence_id: d.sentence_id,
                        text: d.text,
                        translation_en: d.en.filter(|s| !s.is_empty()),
                        translation_ru: d.ru.filter(|s| !s.is_empty()),
                        translation_vi: d.vi.filter(|s| !s.is_empty()),
                        translation_ko: d.ko.filter(|s| !s.is_empty()),
                        ruby: d
                            .ruby
                            .into_iter()
                            .map(|(surface, reading)| RubyPair { surface, reading })
                            .collect(),
                    },
                );
            }
        }
    })
}

pub fn get_cached_example_detail(sentence_id: u32) -> Option<ExampleDetail> {
    let guard = cache().read().ok()?;
    guard.as_ref()?.details.get(&sentence_id).cloned()
}

pub fn is_example_chunk_loaded(chunk_id: u32) -> bool {
    let Ok(guard) = cache().read() else {
        return false;
    };
    guard
        .as_ref()
        .is_some_and(|c| c.loaded_chunks.contains(&chunk_id))
}

/// Test-only reset; keeps tests independent from global state.
pub fn reset_example_data_for_test() {
    if let Ok(mut guard) = cache().write() {
        *guard = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CHUNK_JSON: &str = r#"[
        {"i":0,"x":"家族は４人です。","en":"We are a family of four.","ru":"Нас четверо.","vi":"Gia đình tôi có bốn người.","ko":"가족은 네 명입니다.","f":[["家族","かぞく"]]},
        {"i":1,"x":"私は忙しい。","en":"I'm busy."}
    ]"#;

    // Distinct chunk per test: parallel tests share the global cache, and
    // reinstalling the same chunk is a no-op (ids land in different chunks).
    const REINSTALL_JSON: &str = r#"[
        {"i":3000,"x":"犬です。","en":"It's a dog."}
    ]"#;

    const FALLBACK_JSON: &str = r#"[
        {"i":1000,"x":"私は忙しい。","en":"I'm busy."}
    ]"#;

    const EMPTY_JSON: &str = r#"[{"i":2000,"x":"語。","en":"","ru":""}]"#;

    #[test]
    fn chunk_install_and_lookup() {
        assert!(!is_example_chunk_loaded(0)); // fresh process: nothing installed yet
        cache_example_details(CHUNK_JSON).expect("install");
        assert!(is_example_chunk_loaded(0));
        let d = get_cached_example_detail(0).expect("detail");
        assert_eq!(d.text, "家族は４人です。");
        assert_eq!(d.translation_ru.as_deref(), Some("Нас четверо."));
        assert_eq!(d.ruby.len(), 1);
        assert_eq!(d.ruby[0].surface, "家族");
        assert_eq!(d.ruby[0].reading, "かぞく");
    }

    #[test]
    fn reinstall_is_noop_keeping_first_data() {
        cache_example_details(REINSTALL_JSON).expect("install");
        cache_example_details(REINSTALL_JSON).expect("second install no-op");
        let d = get_cached_example_detail(3000).expect("detail kept from first install");
        assert_eq!(d.text, "犬です。");
        assert!(is_example_chunk_loaded(3));
    }

    #[test]
    fn missing_translations_fall_back_to_english() {
        use crate::domain::value_objects::NativeLanguage;
        cache_example_details(FALLBACK_JSON).expect("install");
        let d = get_cached_example_detail(1000).expect("detail");
        assert_eq!(
            d.translation(&NativeLanguage::Vietnamese),
            Some("I'm busy.")
        );
        assert_eq!(d.translation(&NativeLanguage::Russian), Some("I'm busy."));
        assert_eq!(d.translation_ru, None);
    }

    #[test]
    fn empty_translation_strings_are_treated_as_missing() {
        use crate::domain::value_objects::NativeLanguage;
        cache_example_details(EMPTY_JSON).expect("install");
        let d = get_cached_example_detail(2000).expect("detail");
        assert_eq!(d.translation(&NativeLanguage::English), None);
        assert_eq!(d.translation_ru, None);
    }

    #[test]
    fn corrupt_chunk_json_is_an_error() {
        assert!(cache_example_details("[][").is_err());
    }
}
