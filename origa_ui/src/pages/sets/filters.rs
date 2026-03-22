use origa::domain::{JapaneseLevel, NativeLanguage};
use origa::traits::{TypeMeta, get_types_meta};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum LevelFilter {
    #[default]
    All,
    N5,
    N4,
    N3,
    N2,
    N1,
}

impl LevelFilter {
    pub fn label(&self) -> &'static str {
        match self {
            LevelFilter::All => "Все уровни",
            LevelFilter::N5 => "N5",
            LevelFilter::N4 => "N4",
            LevelFilter::N3 => "N3",
            LevelFilter::N2 => "N2",
            LevelFilter::N1 => "N1",
        }
    }

    pub fn matches(&self, level: JapaneseLevel) -> bool {
        match self {
            LevelFilter::All => true,
            LevelFilter::N5 => level == JapaneseLevel::N5,
            LevelFilter::N4 => level == JapaneseLevel::N4,
            LevelFilter::N3 => level == JapaneseLevel::N3,
            LevelFilter::N2 => level == JapaneseLevel::N2,
            LevelFilter::N1 => level == JapaneseLevel::N1,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct TypeFilter(pub Option<String>);

impl TypeFilter {
    pub fn all() -> Self {
        Self(None)
    }

    pub fn specific(type_id: &str) -> Self {
        Self(Some(type_id.to_string()))
    }

    pub fn is_all(&self) -> bool {
        self.0.is_none()
    }

    pub fn matches(&self, set_type: &str) -> bool {
        match &self.0 {
            None => true,
            Some(filter) => filter == set_type,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ImportFilter {
    #[default]
    All,
    Imported,
    New,
}

impl ImportFilter {
    pub fn label(&self) -> &'static str {
        match self {
            ImportFilter::All => "Все",
            ImportFilter::Imported => "Импортированые",
            ImportFilter::New => "Новые",
        }
    }

    pub fn matches(&self, is_imported: bool) -> bool {
        match self {
            ImportFilter::All => true,
            ImportFilter::Imported => is_imported,
            ImportFilter::New => !is_imported,
        }
    }
}

pub fn available_set_types() -> Vec<TypeMeta> {
    get_types_meta()
        .map(|meta| meta.types.clone())
        .unwrap_or_default()
}

pub fn set_type_label(id: &String, lang: &NativeLanguage) -> String {
    get_types_meta()
        .map(|meta| meta.get_label(id, lang).to_string())
        .unwrap_or_else(|| id.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn level_filter_default_is_all() {
        assert_eq!(LevelFilter::default(), LevelFilter::All);
    }

    #[test]
    fn type_filter_default_is_all() {
        assert_eq!(TypeFilter::default(), TypeFilter::all());
    }

    #[test]
    fn level_filter_matches_all_returns_true() {
        let filter = LevelFilter::All;
        assert!(filter.matches(JapaneseLevel::N5));
        assert!(filter.matches(JapaneseLevel::N1));
    }

    #[test]
    fn level_filter_matches_specific_level() {
        let filter = LevelFilter::N3;
        assert!(!filter.matches(JapaneseLevel::N5));
        assert!(filter.matches(JapaneseLevel::N3));
        assert!(!filter.matches(JapaneseLevel::N1));
    }

    #[test]
    fn type_filter_all_returns_true_for_all_types() {
        let filter = TypeFilter::all();
        assert!(filter.matches("Jlpt"));
        assert!(filter.matches("Migii"));
        assert!(filter.matches("SpyFamily"));
        assert!(filter.matches("DuolingoRu"));
        assert!(filter.matches("DuolingoEn"));
        assert!(filter.matches("MinnaNoNihongo"));
    }

    #[test]
    fn type_filter_specific_matches_only_that_type() {
        let filter = TypeFilter::specific("Migii");
        assert!(!filter.matches("Jlpt"));
        assert!(filter.matches("Migii"));
        assert!(!filter.matches("SpyFamily"));
        assert!(!filter.matches("DuolingoRu"));
        assert!(!filter.matches("DuolingoEn"));
        assert!(!filter.matches("MinnaNoNihongo"));
    }

    #[test]
    fn type_filter_is_all() {
        assert!(TypeFilter::all().is_all());
        assert!(!TypeFilter::specific("Jlpt").is_all());
    }

    #[test]
    fn type_filter_specific_constructor() {
        let filter = TypeFilter::specific("Jlpt");
        assert!(!filter.is_all());
        assert!(filter.matches("Jlpt"));
        assert!(!filter.matches("Migii"));
    }

    #[test]
    fn level_filter_labels() {
        assert_eq!(LevelFilter::All.label(), "Все уровни");
        assert_eq!(LevelFilter::N5.label(), "N5");
        assert_eq!(LevelFilter::N1.label(), "N1");
    }

    #[test]
    fn import_filter_default_is_all() {
        assert_eq!(ImportFilter::default(), ImportFilter::All);
    }

    #[test]
    fn import_filter_matches_all_returns_true() {
        let filter = ImportFilter::All;
        assert!(filter.matches(true));
        assert!(filter.matches(false));
    }

    #[test]
    fn import_filter_matches_imported() {
        let filter = ImportFilter::Imported;
        assert!(filter.matches(true));
        assert!(!filter.matches(false));
    }

    #[test]
    fn import_filter_matches_new() {
        let filter = ImportFilter::New;
        assert!(!filter.matches(true));
        assert!(filter.matches(false));
    }

    #[test]
    fn import_filter_labels() {
        assert_eq!(ImportFilter::All.label(), "Все");
        assert_eq!(ImportFilter::Imported.label(), "Импортированые");
        assert_eq!(ImportFilter::New.label(), "Новые");
    }

    #[test]
    fn available_set_types_returns_empty_without_meta() {
        // get_types_meta() returns None by default
        let types = available_set_types();
        assert!(types.is_empty());
    }

    #[test]
    fn set_type_label_fallback_to_id() {
        // When get_types_meta() returns None, fallback to id
        assert_eq!(
            set_type_label(&"Jlpt".to_string(), &NativeLanguage::Russian),
            "Jlpt"
        );
        assert_eq!(
            set_type_label(&"Migii".to_string(), &NativeLanguage::English),
            "Migii"
        );
    }
}
