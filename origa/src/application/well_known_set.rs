use std::future::Future;

use serde::{Deserialize, Serialize};

use crate::domain::{JapaneseLevel, NativeLanguage, OrigaError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SetType {
    Jlpt,
    Migii,
}

impl SetType {
    pub fn label(&self) -> &'static str {
        match self {
            SetType::Jlpt => "JLPT",
            SetType::Migii => "Migii",
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

pub fn id_to_path(id: &str) -> String {
    if let Some(level) = id.strip_prefix("jlpt_") {
        format!("domain/well_known_set/jltp_{}.json", level)
    } else if let Some(rest) = id.strip_prefix("migii_") {
        let level = rest.split('_').next().unwrap_or("");
        format!("domain/well_known_set/migii/{}/{}.json", level, id)
    } else {
        format!("domain/well_known_set/{}.json", id)
    }
}

pub struct WellKnownSet {
    level: JapaneseLevel,
    words: Vec<String>,
}

impl WellKnownSet {
    pub fn words(&self) -> &[String] {
        &self.words
    }

    pub fn level(&self) -> &JapaneseLevel {
        &self.level
    }
}

pub trait WellKnownSetLoader {
    fn load_meta_list(
        &self,
    ) -> impl Future<Output = Result<Vec<WellKnownSetMeta>, OrigaError>> + Send;
    fn load_set(&self, id: String)
        -> impl Future<Output = Result<WellKnownSet, OrigaError>> + Send;
}
