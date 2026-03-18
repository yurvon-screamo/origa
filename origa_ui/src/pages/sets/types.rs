#[derive(Clone, PartialEq)]
pub struct PreviewWord {
    pub word: String,
    pub meaning: Option<String>,
    pub is_known: bool,
    pub set_id: String,
    pub set_title: String,
}

#[derive(Clone, PartialEq)]
pub struct SetInfo {
    pub set_id: String,
    pub title: String,
    pub description: String,
    pub word_count: Option<usize>,
    pub set_type: origa::traits::SetType,
    pub level: origa::domain::JapaneseLevel,
    pub is_imported: bool,
}
