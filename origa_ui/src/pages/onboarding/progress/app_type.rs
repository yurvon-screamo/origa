use origa::domain::JapaneseLevel;

#[derive(Clone)]
pub enum AppType {
    DuolingoRu,
    DuolingoEn,
    Migii,
    MinnaNoNihongoN5,
    MinnaNoNihongoN4,
}

pub fn parse_app_type(app_id: &str) -> Option<AppType> {
    match app_id {
        "DuolingoRu" => Some(AppType::DuolingoRu),
        "DuolingoEn" => Some(AppType::DuolingoEn),
        "Migii" => Some(AppType::Migii),
        "MinnaNoNihongoN5" => Some(AppType::MinnaNoNihongoN5),
        "MinnaNoNihongoN4" => Some(AppType::MinnaNoNihongoN4),
        _ => None,
    }
}

pub fn level_to_str(level: JapaneseLevel) -> &'static str {
    match level {
        JapaneseLevel::N5 => "N5",
        JapaneseLevel::N4 => "N4",
        JapaneseLevel::N3 => "N3",
        JapaneseLevel::N2 => "N2",
        JapaneseLevel::N1 => "N1",
    }
}
