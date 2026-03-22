use std::future::Future;
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

use crate::domain::{JapaneseLevel, NativeLanguage, OrigaError};

pub type SetType = String;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypeMeta {
    pub id: String,
    pub label_ru: String,
    pub label_en: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypesMeta {
    pub types: Vec<TypeMeta>,
}

impl TypesMeta {
    pub fn get_label<'a>(&'a self, set_type: &'a SetType, lang: &NativeLanguage) -> &'a str {
        self.types
            .iter()
            .find(|t| &t.id == set_type)
            .map(|t| match lang {
                NativeLanguage::Russian => t.label_ru.as_str(),
                NativeLanguage::English => t.label_en.as_str(),
            })
            .unwrap_or(set_type)
    }
}

static TYPES_META_CACHE: OnceLock<TypesMeta> = OnceLock::new();

pub fn set_types_meta(meta: TypesMeta) {
    let _ = TYPES_META_CACHE.set(meta);
}

pub fn get_types_meta() -> Option<&'static TypesMeta> {
    TYPES_META_CACHE.get()
}

pub fn id_to_set_type(id: &str) -> SetType {
    if let Some(meta) = get_types_meta() {
        for t in &meta.types {
            if id.contains(&t.id) || id.starts_with(&t.id.to_lowercase()) {
                return t.id.clone();
            }
        }
    }

    if id.starts_with("jlpt_") {
        "Jlpt".to_string()
    } else if id.starts_with("migii_") {
        "Migii".to_string()
    } else if id.starts_with("spy_family") {
        "SpyFamily".to_string()
    } else if id.contains("_en_") {
        "DuolingoEn".to_string()
    } else if id.contains("_ru_") || id.starts_with("duolingo_") {
        "DuolingoRu".to_string()
    } else if id.starts_with("minna_n5_") {
        "MinnaNoNihongo".to_string()
    } else {
        id.split('_').next().unwrap_or("Unknown").to_string()
    }
}

pub fn resolve_set_path(id: &str) -> String {
    if id.contains("..") || id.contains('/') {
        return format!("domain/well_known_set/{}.json", id);
    }

    if let Some(level) = id.strip_prefix("jlpt_") {
        format!("domain/well_known_set/jlpt_{}.json", level)
    } else if let Some(rest) = id.strip_prefix("migii_") {
        let level = rest.split('_').next().unwrap_or("");
        format!("domain/well_known_set/migii/{}/{}.json", level, id)
    } else if let Some(rest) = id.strip_prefix("duolingo_") {
        let level = rest.split('_').next().unwrap_or("");
        let parts: Vec<&str> = rest.split('_').collect();
        let filename = if parts.len() >= 4 {
            format!("{}_{}", parts[2], parts[3])
        } else {
            rest.to_string()
        };
        format!("domain/well_known_set/duolingo/{}/{}.json", level, filename)
    } else if id.starts_with("minna_n5_") {
        format!("domain/well_known_set/minna_n5/{}.json", id)
    } else {
        format!("domain/well_known_set/{}.json", id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WellKnownSetMeta {
    pub id: String,
    #[serde(rename = "set_type")]
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

    pub fn set_type_label(&self, lang: &NativeLanguage) -> String {
        get_types_meta()
            .map(|m| m.get_label(&self.set_type, lang).to_string())
            .unwrap_or_else(|| self.set_type.clone())
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

    fn load_sets(
        &self,
        ids: Vec<String>,
    ) -> impl Future<Output = Result<Vec<(String, WellKnownSet)>, OrigaError>> {
        async {
            let mut results = Vec::new();
            for id in ids {
                let set = self.load_set(id.clone()).await?;
                results.push((id, set));
            }
            Ok(results)
        }
    }
}
