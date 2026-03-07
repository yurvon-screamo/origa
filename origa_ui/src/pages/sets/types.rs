#[derive(Clone, PartialEq)]
pub struct SetInfo {
    pub set_id: String,
    pub title: String,
    pub description: String,
    pub word_count: Option<usize>,
    pub set_type: origa::application::SetType,
    pub level: origa::domain::JapaneseLevel,
    pub is_imported: bool,
}
