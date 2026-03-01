#[derive(Clone, Debug, PartialEq)]
pub struct ImportState {
    pub set_id: String,
    pub title: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ImportResult {
    pub is_success: bool,
    pub message: String,
}

#[derive(Clone, PartialEq)]
pub struct SetInfo {
    pub set_id: String,
    pub title: String,
    pub description: String,
    pub word_count: Option<usize>,
    pub set_type: origa::application::SetType,
    pub level: origa::domain::JapaneseLevel,
}
