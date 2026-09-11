//! `VocabularyInfo` — the per-word translation record shared by the owned
//! database and the archived CDN blob.

use crate::domain::value_objects::NativeLanguage;

#[derive(Debug, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct VocabularyInfo {
    pub(super) word: String,
    pub(super) ru_translations: Vec<String>,
    pub(super) ru_description: Option<String>,
    pub(super) en_translations: Vec<String>,
    pub(super) en_description: Option<String>,
    pub(super) vi_translations: Vec<String>,
    pub(super) vi_description: Option<String>,
    pub(super) ko_translations: Vec<String>,
    pub(super) ko_description: Option<String>,
}

impl VocabularyInfo {
    pub fn word(&self) -> &str {
        &self.word
    }

    pub fn russian_translation(&self) -> String {
        self.bullets(&self.ru_translations)
    }

    pub fn english_translation(&self) -> String {
        self.bullets(&self.en_translations)
    }

    pub fn vietnamese_translation(&self) -> String {
        self.project(&self.vi_translations, Self::english_translation)
    }

    pub fn korean_translation(&self) -> String {
        self.project(&self.ko_translations, Self::english_translation)
    }

    /// Formatted bullets for `primary`, degrading to `fallback` when the
    /// primary list is empty (legacy chunks without vi/ko fields).
    fn project(&self, primary: &[String], fallback: fn(&Self) -> String) -> String {
        if primary.is_empty() {
            fallback(self)
        } else {
            self.bullets(primary)
        }
    }

    fn bullets(&self, list: &[String]) -> String {
        list.iter()
            .map(|t| format!("- {}", t))
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn ru_translations(&self) -> &[String] {
        &self.ru_translations
    }

    pub fn en_translations(&self) -> &[String] {
        &self.en_translations
    }

    pub fn ru_description(&self) -> Option<&str> {
        self.ru_description.as_deref()
    }

    pub fn en_description(&self) -> Option<&str> {
        self.en_description.as_deref()
    }

    pub fn vi_translations(&self) -> &[String] {
        &self.vi_translations
    }

    pub fn ko_translations(&self) -> &[String] {
        &self.ko_translations
    }

    pub fn translations(&self, lang: &NativeLanguage) -> &[String] {
        // KO/VI fall back to English for legacy chunks without vi/ko fields.
        match lang {
            NativeLanguage::Russian => &self.ru_translations,
            NativeLanguage::English => &self.en_translations,
            NativeLanguage::Korean => self.pick(&self.ko_translations),
            NativeLanguage::Vietnamese => self.pick(&self.vi_translations),
        }
    }

    pub fn description(&self, lang: &NativeLanguage) -> Option<&str> {
        match lang {
            NativeLanguage::Russian => self.ru_description.as_deref(),
            NativeLanguage::English => self.en_description.as_deref(),
            NativeLanguage::Korean => self.pick_desc(&self.ko_description),
            NativeLanguage::Vietnamese => self.pick_desc(&self.vi_description),
        }
    }

    fn pick<'a>(&'a self, primary: &'a [String]) -> &'a [String] {
        if primary.is_empty() {
            &self.en_translations
        } else {
            primary
        }
    }

    fn pick_desc<'a>(&'a self, primary: &'a Option<String>) -> Option<&'a str> {
        primary.as_deref().or(self.en_description.as_deref())
    }
}
