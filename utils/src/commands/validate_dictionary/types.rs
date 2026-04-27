use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ValidationRecord {
    pub word: String,
    pub valid: bool,
    pub raw_response: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ValidationSummary {
    pub total_checked: usize,
    pub total_valid: usize,
    pub total_invalid: usize,
    pub invalid_words: Vec<String>,
}
