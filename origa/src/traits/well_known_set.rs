use std::future::Future;

use serde::{Deserialize, Serialize};

use crate::domain::{JapaneseLevel, NativeLanguage, OrigaError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SetType {
    Jlpt,
    Migii,
    SpyFamily,
    DuolingoRu,
    DuolingoEn,
}

impl SetType {
    pub fn label(&self) -> &'static str {
        match self {
            SetType::Jlpt => "JLPT",
            SetType::Migii => "Migii",
            SetType::SpyFamily => "SpyFamily",
            SetType::DuolingoRu => "Duolingo Ru",
            SetType::DuolingoEn => "Duolingo En",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WellKnownSetMeta {
    pub id: String,
    pub set_type: SetType,
    pub level: JapaneseLevel,
    pub title_ru: String,
    pub title_en: String,
    pub desc_ru: String,
    pub desc_en: String,
    #[serde(default)]
    pub word_count: usize,
}

impl WellKnownSetMeta {
    pub fn title(&self, lang: &NativeLanguage) -> &str {
        match lang {
            NativeLanguage::Russian => &self.title_ru,
            NativeLanguage::English => &self.title_en,
        }
    }

    pub fn description(&self, lang: &NativeLanguage) -> &str {
        match lang {
            NativeLanguage::Russian => &self.desc_ru,
            NativeLanguage::English => &self.desc_en,
        }
    }
}

#[derive(Clone)]
pub struct WellKnownSet {
    level: JapaneseLevel,
    words: Vec<String>,
}

impl WellKnownSet {
    pub fn new(level: JapaneseLevel, words: Vec<String>) -> Self {
        Self { level, words }
    }

    pub fn words(&self) -> &[String] {
        &self.words
    }

    pub fn level(&self) -> &JapaneseLevel {
        &self.level
    }
}

pub trait WellKnownSetLoader {
    fn load_meta_list(&self) -> impl Future<Output = Result<Vec<WellKnownSetMeta>, OrigaError>>;
    fn load_set(&self, id: String) -> impl Future<Output = Result<WellKnownSet, OrigaError>>;
}
