use origa::domain::JapaneseLevel;
use origa::traits::SetType;

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

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum TypeFilter {
    #[default]
    All,
    Jlpt,
    Migii,
    SpyFamily,
    DuolingoRu,
    DuolingoEn,
}

impl TypeFilter {
    pub fn label(&self) -> &'static str {
        match self {
            TypeFilter::All => "Все типы",
            TypeFilter::Jlpt => "JLPT",
            TypeFilter::Migii => "Migii",
            TypeFilter::SpyFamily => "SpyFamily",
            TypeFilter::DuolingoRu => "Duolingo Ru",
            TypeFilter::DuolingoEn => "Duolingo En",
        }
    }

    pub fn matches(&self, set_type: SetType) -> bool {
        match self {
            TypeFilter::All => true,
            TypeFilter::Jlpt => set_type == SetType::Jlpt,
            TypeFilter::Migii => set_type == SetType::Migii,
            TypeFilter::SpyFamily => set_type == SetType::SpyFamily,
            TypeFilter::DuolingoRu => set_type == SetType::DuolingoRu,
            TypeFilter::DuolingoEn => set_type == SetType::DuolingoEn,
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

#[cfg(test)]
mod tests {
    use super::*;
    use origa::domain::JapaneseLevel;
    use origa::traits::SetType;

    #[test]
    fn level_filter_default_is_all() {
        assert_eq!(LevelFilter::default(), LevelFilter::All);
    }

    #[test]
    fn type_filter_default_is_all() {
        assert_eq!(TypeFilter::default(), TypeFilter::All);
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
    fn type_filter_matches_all_returns_true() {
        let filter = TypeFilter::All;
        assert!(filter.matches(SetType::Jlpt));
        assert!(filter.matches(SetType::Migii));
        assert!(filter.matches(SetType::SpyFamily));
        assert!(filter.matches(SetType::DuolingoRu));
        assert!(filter.matches(SetType::DuolingoEn));
    }

    #[test]
    fn type_filter_matches_specific_type() {
        let filter = TypeFilter::Migii;
        assert!(!filter.matches(SetType::Jlpt));
        assert!(filter.matches(SetType::Migii));
        assert!(!filter.matches(SetType::SpyFamily));
        assert!(!filter.matches(SetType::DuolingoRu));
        assert!(!filter.matches(SetType::DuolingoEn));
    }

    #[test]
    fn level_filter_labels() {
        assert_eq!(LevelFilter::All.label(), "Все уровни");
        assert_eq!(LevelFilter::N5.label(), "N5");
        assert_eq!(LevelFilter::N1.label(), "N1");
    }

    #[test]
    fn type_filter_labels() {
        assert_eq!(TypeFilter::All.label(), "Все типы");
        assert_eq!(TypeFilter::Jlpt.label(), "JLPT");
        assert_eq!(TypeFilter::Migii.label(), "Migii");
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
}
