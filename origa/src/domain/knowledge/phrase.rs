use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::dictionary::phrase::{get_phrase_text, get_phrase_translation};
use crate::domain::value_objects::NativeLanguage;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PhraseCard {
    phrase_id: Ulid,
}

impl PhraseCard {
    pub fn new(phrase_id: Ulid) -> Self {
        Self { phrase_id }
    }

    pub fn phrase_id(&self) -> &Ulid {
        &self.phrase_id
    }

    pub fn question(&self) -> Option<String> {
        get_phrase_text(&self.phrase_id)
    }

    pub fn answer(&self, lang: &NativeLanguage) -> Option<String> {
        get_phrase_translation(&self.phrase_id, lang).or_else(|| get_phrase_text(&self.phrase_id))
    }

    #[cfg(test)]
    pub fn new_test_with_id(phrase_id: Ulid) -> Self {
        Self { phrase_id }
    }
}
