use crate::i18n::Locale;
use leptos_i18n::I18nContext;
use origa::domain::JapaneseLevel;
use origa::domain::{TypeMeta, get_types_meta};

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
    pub fn label(&self, i18n: &I18nContext<Locale>) -> String {
        match self {
            LevelFilter::All => i18n.get_keys().sets().all_levels().inner().to_string(),
            LevelFilter::N5 => "N5".to_string(),
            LevelFilter::N4 => "N4".to_string(),
            LevelFilter::N3 => "N3".to_string(),
            LevelFilter::N2 => "N2".to_string(),
            LevelFilter::N1 => "N1".to_string(),
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
    pub fn label(&self, i18n: &I18nContext<Locale>) -> String {
        match self {
            ImportFilter::All => i18n.get_keys().sets().all().inner().to_string(),
            ImportFilter::Imported => i18n.get_keys().sets().filter_imported().inner().to_string(),
            ImportFilter::New => i18n.get_keys().sets().filter_new_sets().inner().to_string(),
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
    fn available_set_types_returns_empty_without_meta() {
        let types = available_set_types();
        assert!(types.is_empty());
    }
}
