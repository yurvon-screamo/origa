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
        let remaining = parts.iter().skip(1).copied().collect::<Vec<_>>().join("_");
        let filename = if remaining.starts_with("duolingo_") {
            remaining
        } else if parts.len() >= 3 {
            parts[2..].join("_")
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

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("jlpt_n5", "Jlpt")]
    #[case("jlpt_n4", "Jlpt")]
    #[case("jlpt_n3", "Jlpt")]
    #[case("jlpt_n2", "Jlpt")]
    #[case("jlpt_n1", "Jlpt")]
    #[case("migii_n5_1", "Migii")]
    #[case("migii_n4_grammar", "Migii")]
    #[case("spy_family_s1", "SpyFamily")]
    #[case("spy_family_1", "SpyFamily")]
    #[case("word_en_1", "DuolingoEn")]
    #[case("lesson_en_01", "DuolingoEn")]
    #[case("word_ru_1", "DuolingoRu")]
    #[case("lesson_ru_01", "DuolingoRu")]
    #[case("duolingo_n5_animals", "DuolingoRu")]
    #[case("duolingo_n4_verbs", "DuolingoRu")]
    #[case("minna_n5_01", "MinnaNoNihongo")]
    #[case("minna_n4_02", "minna")]
    #[case("unknown_id", "unknown")]
    #[case("some_random_set", "some")]
    fn test_id_to_set_type_various_prefixes(#[case] id: &str, #[case] expected: &str) {
        assert_eq!(id_to_set_type(id), expected);
    }

    #[rstest]
    #[case("jlpt_n5", "domain/well_known_set/jlpt_n5.json")]
    #[case("jlpt_n4", "domain/well_known_set/jlpt_n4.json")]
    #[case("migii_n5_basic", "domain/well_known_set/migii/n5/migii_n5_basic.json")]
    #[case(
        "migii_n4_grammar",
        "domain/well_known_set/migii/n4/migii_n4_grammar.json"
    )]
    #[case(
        "duolingo_n5_animals",
        "domain/well_known_set/duolingo/n5/n5_animals.json"
    )]
    #[case("duolingo_n4_verbs", "domain/well_known_set/duolingo/n4/n4_verbs.json")]
    #[case(
        "duolingo_n5_duolingo_ru_n5_1",
        "domain/well_known_set/duolingo/n5/duolingo_ru_n5_1.json"
    )]
    #[case(
        "duolingo_n4_duolingo_jp_n4_2",
        "domain/well_known_set/duolingo/n4/duolingo_jp_n4_2.json"
    )]
    #[case("minna_n5_01", "domain/well_known_set/minna_n5/minna_n5_01.json")]
    #[case("minna_n4_02", "domain/well_known_set/minna_n4_02.json")]
    #[case("spy_family_s1", "domain/well_known_set/spy_family_s1.json")]
    #[case("random_id", "domain/well_known_set/random_id.json")]
    fn test_resolve_set_path_formats(#[case] id: &str, #[case] expected: &str) {
        assert_eq!(resolve_set_path(id), expected);
    }

    #[test]
    fn test_resolve_set_path_handles_special_characters() {
        let id_with_dots = "test..dots";
        let path = resolve_set_path(id_with_dots);
        assert!(path.starts_with("domain/well_known_set/"));
        assert!(path.ends_with(".json"));
    }

    #[test]
    fn test_types_meta_get_label_returns_correct_labels() {
        let meta = TypesMeta {
            types: vec![TypeMeta {
                id: "Test".to_string(),
                label_ru: "Тестовая метка".to_string(),
                label_en: "Test Label".to_string(),
            }],
        };

        assert_eq!(
            meta.get_label(&"Test".to_string(), &NativeLanguage::Russian),
            "Тестовая метка"
        );
        assert_eq!(
            meta.get_label(&"Test".to_string(), &NativeLanguage::English),
            "Test Label"
        );
        assert_eq!(
            meta.get_label(&"nonexistent".to_string(), &NativeLanguage::Russian),
            "nonexistent"
        );
    }

    #[test]
    fn test_well_known_set_meta_accessors() {
        let meta = WellKnownSetMeta {
            id: "test_set".to_string(),
            set_type: "Test".to_string(),
            level: JapaneseLevel::N5,
            title_ru: "Тестовый набор".to_string(),
            title_en: "Test Set".to_string(),
            desc_ru: "Тестовое описание".to_string(),
            desc_en: "A test set".to_string(),
            word_count: 0,
        };

        assert_eq!(meta.title(&NativeLanguage::English), "Test Set");
        assert_eq!(meta.title(&NativeLanguage::Russian), "Тестовый набор");
        assert_eq!(meta.description(&NativeLanguage::English), "A test set");
        assert_eq!(
            meta.description(&NativeLanguage::Russian),
            "Тестовое описание"
        );
        assert_eq!(meta.level, JapaneseLevel::N5);
    }

    #[test]
    fn test_well_known_set_accessors() {
        let set = WellKnownSet::new(
            JapaneseLevel::N5,
            vec!["word1".to_string(), "word2".to_string()],
        );

        assert_eq!(set.level(), &JapaneseLevel::N5);
        assert_eq!(set.words().len(), 2);
        assert_eq!(set.words(), &["word1".to_string(), "word2".to_string()]);
    }
}
