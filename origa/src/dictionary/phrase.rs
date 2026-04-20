use std::collections::{HashMap, HashSet};
use std::sync::{OnceLock, RwLock};

use serde::Deserialize;
use ulid::Ulid;

use crate::domain::OrigaError;
use crate::domain::value_objects::NativeLanguage;

pub static PHRASE_INDEX: OnceLock<PhraseIndex> = OnceLock::new();

pub static PHRASE_DATA: OnceLock<RwLock<PhraseDataCache>> = OnceLock::new();

pub struct PhraseIndex {
    entries: HashMap<Ulid, IndexEntry>,
    token_to_phrases: HashMap<String, Vec<Ulid>>,
    all_ids: HashSet<Ulid>,
    version: u32,
    hash: String,
}

pub struct IndexEntry {
    id: Ulid,
    tokens: Vec<String>,
    chunk_id: u32,
}

impl IndexEntry {
    pub fn id(&self) -> &Ulid {
        &self.id
    }

    pub fn tokens(&self) -> &[String] {
        &self.tokens
    }

    pub fn chunk_id(&self) -> u32 {
        self.chunk_id
    }
}

#[derive(Deserialize)]
struct IndexFile {
    #[serde(rename = "v")]
    version: u32,
    #[serde(rename = "h")]
    hash: String,
    #[serde(rename = "phrases")]
    phrases: Vec<IndexEntryRaw>,
}

#[derive(Deserialize)]
struct IndexEntryRaw {
    #[serde(rename = "i")]
    id: Ulid,
    #[serde(rename = "t")]
    tokens: Vec<String>,
    #[serde(rename = "c")]
    chunk_id: u32,
}

impl PhraseIndex {
    fn from_json(json: &str) -> Result<Self, OrigaError> {
        let file: IndexFile =
            serde_json::from_str(json).map_err(|e| OrigaError::PhraseParseError {
                reason: format!("Failed to parse phrase index: {}", e),
            })?;

        let mut entries = HashMap::with_capacity(file.phrases.len());
        let mut token_to_phrases: HashMap<String, Vec<Ulid>> = HashMap::new();

        for raw in file.phrases {
            let id = raw.id;
            for token in &raw.tokens {
                token_to_phrases.entry(token.clone()).or_default().push(id);
            }
            entries.insert(
                id,
                IndexEntry {
                    id,
                    tokens: raw.tokens,
                    chunk_id: raw.chunk_id,
                },
            );
        }

        let all_ids: HashSet<Ulid> = entries.keys().copied().collect();

        Ok(Self {
            entries,
            token_to_phrases,
            all_ids,
            version: file.version,
            hash: file.hash,
        })
    }

    fn get_entry(&self, id: &Ulid) -> Option<&IndexEntry> {
        self.entries.get(id)
    }

    fn get_phrases_by_token(&self, token: &str) -> Vec<&IndexEntry> {
        self.token_to_phrases
            .get(token)
            .map(|ids| ids.iter().filter_map(|id| self.entries.get(id)).collect())
            .unwrap_or_default()
    }

    fn iter_entries(&self) -> impl Iterator<Item = &IndexEntry> {
        self.entries.values()
    }

    fn all_ids(&self) -> &HashSet<Ulid> {
        &self.all_ids
    }
}

#[derive(Clone)]
pub struct PhraseDetail {
    pub id: Ulid,
    pub text: String,
    pub translation_ru: Option<String>,
    pub translation_en: Option<String>,
}

impl PhraseDetail {
    pub fn translation(&self, lang: &crate::domain::value_objects::NativeLanguage) -> Option<&str> {
        match lang {
            crate::domain::value_objects::NativeLanguage::Russian => self.translation_ru.as_deref(),
            crate::domain::value_objects::NativeLanguage::English => self.translation_en.as_deref(),
        }
    }
}

pub struct PhraseDataCache {
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
}

pub fn init_phrase_index(json: &str) -> Result<(), OrigaError> {
    let _ = PHRASE_DATA.set(RwLock::new(PhraseDataCache::new()));

    let index = PhraseIndex::from_json(json)?;
    PHRASE_INDEX
        .set(index)
        .map_err(|_| OrigaError::PhraseParseError {
            reason: "Phrase index already initialized".to_string(),
        })
}

pub fn is_phrases_loaded() -> bool {
    PHRASE_INDEX.get().is_some()
}

pub fn get_phrases_by_token(token: &str) -> Vec<&'static IndexEntry> {
    PHRASE_INDEX
        .get()
        .map(|idx| idx.get_phrases_by_token(token))
        .unwrap_or_default()
}

pub fn get_chunk_id(id: &Ulid) -> Option<u32> {
    PHRASE_INDEX
        .get()
        .and_then(|idx| idx.get_entry(id).map(|e| e.chunk_id))
}

pub fn iter_index_entries() -> Option<impl Iterator<Item = &'static IndexEntry>> {
    PHRASE_INDEX.get().map(|idx| idx.iter_entries())
}

pub fn get_all_index_ids() -> HashSet<Ulid> {
    PHRASE_INDEX
        .get()
        .map(|idx| idx.all_ids().clone())
        .unwrap_or_default()
}

pub fn index_version() -> (u32, String) {
    PHRASE_INDEX
        .get()
        .map(|idx| (idx.version, idx.hash.clone()))
        .unwrap_or((0, String::new()))
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
            translation_ru: r.translation_ru,
            translation_en: r.translation_en,
        })
        .collect();

    let cache = PHRASE_DATA
        .get()
        .ok_or_else(|| OrigaError::PhraseParseError {
            reason: "Phrase data cache not initialized".to_string(),
        })?;

    let mut guard = cache.write().unwrap_or_else(|e| e.into_inner());

    guard.insert_chunk(chunk_id, details);
    Ok(())
}

pub fn get_cached_phrase_detail(id: &Ulid) -> Option<PhraseDetail> {
    let cache = PHRASE_DATA.get()?;
    let guard = cache.read().unwrap_or_else(|e| e.into_inner());
    guard.get_detail(id).cloned()
}

pub fn get_phrase_text(id: &Ulid) -> Option<String> {
    let cache = PHRASE_DATA.get()?;
    let guard = cache.read().unwrap_or_else(|e| e.into_inner());
    guard.get_detail(id).map(|d| d.text.clone())
}

pub fn get_phrase_translation(id: &Ulid, lang: &NativeLanguage) -> Option<String> {
    let cache = PHRASE_DATA.get()?;
    let guard = cache.read().unwrap_or_else(|e| e.into_inner());
    guard.get_detail(id).and_then(|d| match lang {
        NativeLanguage::Russian => d.translation_ru.clone(),
        NativeLanguage::English => d.translation_en.clone(),
    })
}

pub fn is_chunk_loaded(chunk_id: u32) -> bool {
    PHRASE_DATA
        .get()
        .map(|cache| {
            let guard = cache.read().unwrap_or_else(|e| e.into_inner());
            guard.is_chunk_loaded(chunk_id)
        })
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_index_json() -> &'static str {
        r#"{"v":1,"h":"test","total":2,"phrases":[{"i":"01KPJ5S3N1DRFFD236Z4EZ03HJ","t":["hello","world"],"c":0},{"i":"01KPJ5S3N1DRFFD236Z4EZ03HK","t":["goodbye","world"],"c":0}]}"#
    }

    fn test_chunk_json() -> &'static str {
        r#"[{"i":"01KPJ5S3N1DRFFD236Z4EZ03HJ","x":"Hello world","ru":"Привет мир","en":"Hello world"},{"i":"01KPJ5S3N1DRFFD236Z4EZ03HK","x":"Goodbye world","ru":"Прощай мир","en":"Goodbye world"}]"#
    }

    #[test]
    fn init_phrase_index_valid_json() {
        let index = PhraseIndex::from_json(test_index_json()).expect("valid JSON should parse");
        let entries: Vec<_> = index.iter_entries().collect();
        assert_eq!(entries.len(), 2);
        assert_eq!(index.token_to_phrases.len(), 3);
    }

    #[test]
    fn init_phrase_index_invalid_json() {
        let result = PhraseIndex::from_json("not json");
        assert!(result.is_err());
    }

    #[test]
    fn get_entry_found() {
        let index = PhraseIndex::from_json(test_index_json()).expect("valid JSON");
        let id = Ulid::from_string("01KPJ5S3N1DRFFD236Z4EZ03HJ").expect("valid ULID");
        let entry = index.get_entry(&id);
        assert!(entry.is_some());
        assert_eq!(entry.expect("entry").tokens(), &["hello", "world"]);
    }

    #[test]
    fn get_entry_not_found() {
        let index = PhraseIndex::from_json(test_index_json()).expect("valid JSON");
        let missing_id = Ulid::new();
        assert!(index.get_entry(&missing_id).is_none());
    }

    #[test]
    fn get_phrases_by_token_found() {
        let index = PhraseIndex::from_json(test_index_json()).expect("valid JSON");
        let results = index.get_phrases_by_token("world");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn get_phrases_by_token_not_found() {
        let index = PhraseIndex::from_json(test_index_json()).expect("valid JSON");
        let results = index.get_phrases_by_token(" nonexistent");
        assert!(results.is_empty());
    }

    #[test]
    fn all_ids_returns_all() {
        let index = PhraseIndex::from_json(test_index_json()).expect("valid JSON");
        let ids = index.all_ids();
        assert_eq!(ids.len(), 2);
    }

    #[test]
    fn index_version_returns_values() {
        let index = PhraseIndex::from_json(test_index_json()).expect("valid JSON");
        assert_eq!(index.version, 1);
        assert_eq!(index.hash, "test");
    }

    #[test]
    fn cache_phrase_details_works() {
        let mut cache = PhraseDataCache::new();
        let id = Ulid::from_string("01KPJ5S3N1DRFFD236Z4EZ03HJ").expect("valid ULID");

        assert!(!cache.is_chunk_loaded(0));
        assert!(cache.get_detail(&id).is_none());

        let details = vec![PhraseDetail {
            id,
            text: "Hello world".to_string(),
            translation_ru: Some("Привет мир".to_string()),
            translation_en: Some("Hello world".to_string()),
        }];
        cache.insert_chunk(0, details);

        assert!(cache.is_chunk_loaded(0));
        let detail = cache.get_detail(&id).expect("detail should exist");
        assert_eq!(detail.text, "Hello world");
    }

    #[test]
    fn parse_chunk_json() {
        let raw_list: Vec<DetailRaw> =
            serde_json::from_str(test_chunk_json()).expect("valid chunk JSON");
        assert_eq!(raw_list.len(), 2);
        assert_eq!(raw_list[0].text, "Hello world");
        assert_eq!(raw_list[0].translation_ru, Some("Привет мир".to_string()));
        assert!(raw_list[0].translation_en.is_some());
    }
}
