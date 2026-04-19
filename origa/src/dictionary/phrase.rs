use std::{collections::HashMap, sync::OnceLock};

use serde::Deserialize;
use ulid::Ulid;

use crate::domain::OrigaError;
use crate::domain::value_objects::NativeLanguage;

pub static PHRASE_DATABASE: OnceLock<PhraseDatabase> = OnceLock::new();

#[derive(Debug, Clone, Deserialize)]
pub struct PhraseEntry {
    id: Ulid,
    text: String,
    audio_file: String,
    tokens: Vec<String>,
    #[serde(default)]
    translation_ru: Option<String>,
    #[serde(default)]
    translation_en: Option<String>,
}

impl PhraseEntry {
    pub fn id(&self) -> &Ulid {
        &self.id
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn audio_file(&self) -> &str {
        &self.audio_file
    }

    pub fn tokens(&self) -> &[String] {
        &self.tokens
    }

    pub fn translation(&self, lang: &NativeLanguage) -> Option<&str> {
        match lang {
            NativeLanguage::Russian => self.translation_ru.as_deref(),
            NativeLanguage::English => self.translation_en.as_deref(),
        }
    }

    fn normalize_audio_file(&mut self) {
        if self.audio_file.ends_with(".mp3") {
            self.audio_file = self.audio_file.replace(".mp3", ".opus");
        }
    }
}

#[derive(Debug)]
pub struct PhraseDatabase {
    phrases: HashMap<Ulid, PhraseEntry>,
    token_to_phrases: HashMap<String, Vec<Ulid>>,
}

#[derive(Deserialize)]
struct PhraseDataset {
    phrases: Vec<PhraseEntry>,
}

impl PhraseDatabase {
    fn from_json(json: &str) -> Result<Self, OrigaError> {
        let dataset = serde_json::from_str::<PhraseDataset>(json).map_err(|e| {
            OrigaError::PhraseParseError {
                reason: format!("Failed to parse phrase dataset: {}", e),
            }
        })?;

        let mut phrases = HashMap::with_capacity(dataset.phrases.len());
        let mut token_to_phrases: HashMap<String, Vec<Ulid>> = HashMap::new();

        for mut entry in dataset.phrases {
            entry.normalize_audio_file();
            for token in &entry.tokens {
                token_to_phrases
                    .entry(token.clone())
                    .or_default()
                    .push(entry.id);
            }
            let id = entry.id;
            phrases.insert(id, entry);
        }

        Ok(Self {
            phrases,
            token_to_phrases,
        })
    }

    fn get_phrase_by_id(&self, id: &Ulid) -> Option<&PhraseEntry> {
        self.phrases.get(id)
    }

    fn get_phrases_by_token(&self, token: &str) -> Vec<&PhraseEntry> {
        self.token_to_phrases
            .get(token)
            .map(|ids| ids.iter().filter_map(|id| self.phrases.get(id)).collect())
            .unwrap_or_default()
    }

    fn iter_phrases(&self) -> impl Iterator<Item = &PhraseEntry> {
        self.phrases.values()
    }
}

pub fn init_phrases(json: &str) -> Result<(), OrigaError> {
    let db = PhraseDatabase::from_json(json)?;
    PHRASE_DATABASE
        .set(db)
        .map_err(|_| OrigaError::PhraseParseError {
            reason: "Phrase database already initialized".to_string(),
        })
}

pub fn is_phrases_loaded() -> bool {
    PHRASE_DATABASE.get().is_some()
}

pub fn get_phrase_by_id(id: &Ulid) -> Option<&'static PhraseEntry> {
    PHRASE_DATABASE.get().and_then(|db| db.get_phrase_by_id(id))
}

pub fn get_phrases_by_token(token: &str) -> Vec<&'static PhraseEntry> {
    PHRASE_DATABASE
        .get()
        .map(|db| db.get_phrases_by_token(token))
        .unwrap_or_default()
}

pub fn iter_phrases() -> Option<impl Iterator<Item = &'static PhraseEntry>> {
    PHRASE_DATABASE.get().map(|db| db.iter_phrases())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_json_single() -> &'static str {
        r#"{"phrases":[{"id":"01KPJ5S3N1DRFFD236Z4EZ03HJ","text":"こんにちは世界","audio_file":"01KPJ5S3N1DRFFD236Z4EZ03HJ.mp3","tokens":["こんにちは","世界"],"translation_ru":"Привет мир","translation_en":"Hello world"}]}"#
    }

    fn test_json_multiple() -> &'static str {
        r#"{"phrases":[
            {"id":"01KPJ5S3N1DRFFD236Z4EZ03HJ","text":"こんにちは世界","audio_file":"01KPJ5S3N1DRFFD236Z4EZ03HJ.mp3","tokens":["こんにちは","世界"],"translation_ru":"Привет мир","translation_en":"Hello world"},
            {"id":"01KPJ5S3N1DRFFD236Z4EZ03HK","text":"さようなら世界","audio_file":"01KPJ5S3N1DRFFD236Z4EZ03HK.mp3","tokens":["さようなら","世界"],"translation_ru":"Прощай мир","translation_en":"Goodbye world"}
        ]}"#
    }

    fn test_json_without_translations() -> &'static str {
        r#"{"phrases":[{"id":"01KPJ5S3N1DRFFD236Z4EZ03HJ","text":"こんにちは世界","audio_file":"01KPJ5S3N1DRFFD236Z4EZ03HJ.mp3","tokens":["こんにちは","世界"]}]}"#
    }

    fn fresh_db(json: &str) -> PhraseDatabase {
        PhraseDatabase::from_json(json).expect("valid JSON should parse")
    }

    #[test]
    fn init_phrases_valid_json() {
        let db = fresh_db(test_json_single());
        assert_eq!(db.phrases.len(), 1);
        assert_eq!(db.token_to_phrases.len(), 2);
    }

    #[test]
    fn init_phrases_invalid_json() {
        let result = PhraseDatabase::from_json("not json");
        assert!(result.is_err());
    }

    #[test]
    fn get_phrase_by_id_found() {
        let db = fresh_db(test_json_single());
        let id = Ulid::from_string("01KPJ5S3N1DRFFD236Z4EZ03HJ").expect("valid ULID");
        let entry = db.get_phrase_by_id(&id);
        assert!(entry.is_some());
        assert_eq!(entry.expect("entry").text(), "こんにちは世界");
    }

    #[test]
    fn get_phrase_by_id_not_found() {
        let db = fresh_db(test_json_single());
        let missing_id = Ulid::new();
        assert!(db.get_phrase_by_id(&missing_id).is_none());
    }

    #[test]
    fn get_phrases_by_token_found() {
        let db = fresh_db(test_json_single());
        let results = db.get_phrases_by_token("こんにちは");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].text(), "こんにちは世界");
    }

    #[test]
    fn get_phrases_by_token_not_found() {
        let db = fresh_db(test_json_single());
        let results = db.get_phrases_by_token("存在しない");
        assert!(results.is_empty());
    }

    #[test]
    fn get_phrases_by_token_multiple() {
        let db = fresh_db(test_json_multiple());
        let results = db.get_phrases_by_token("世界");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn iter_phrases_count() {
        let db = fresh_db(test_json_multiple());
        assert_eq!(db.iter_phrases().count(), 2);
    }

    #[test]
    fn translation_returns_correct_lang() {
        let db = fresh_db(test_json_single());
        let id = Ulid::from_string("01KPJ5S3N1DRFFD236Z4EZ03HJ").unwrap();
        let entry = db.get_phrase_by_id(&id).unwrap();
        assert_eq!(
            entry.translation(&NativeLanguage::Russian),
            Some("Привет мир")
        );
        assert_eq!(
            entry.translation(&NativeLanguage::English),
            Some("Hello world")
        );
    }

    #[test]
    fn translation_returns_none_when_missing() {
        let db = fresh_db(test_json_without_translations());
        let id = Ulid::from_string("01KPJ5S3N1DRFFD236Z4EZ03HJ").unwrap();
        let entry = db.get_phrase_by_id(&id).unwrap();
        assert_eq!(entry.translation(&NativeLanguage::Russian), None);
        assert_eq!(entry.translation(&NativeLanguage::English), None);
    }

    #[test]
    fn audio_file_mp3_replaced_with_opus() {
        let db = fresh_db(test_json_single());
        let id = Ulid::from_string("01KPJ5S3N1DRFFD236Z4EZ03HJ").unwrap();
        let entry = db.get_phrase_by_id(&id).unwrap();
        assert_eq!(entry.audio_file(), "01KPJ5S3N1DRFFD236Z4EZ03HJ.opus");
    }
}
