//! #528: an SRS card for a textbook example sentence. Created lazily on
//! the learner's first self-assessment of a shown example; lives by its
//! own FSRS schedule afterwards («like phrases», owner design). The word
//! owner's own schedule is never touched.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExampleCard {
    word: String,
    sentence_id: u32,
}

impl ExampleCard {
    pub fn new(word: impl Into<String>, sentence_id: u32) -> Self {
        Self {
            word: word.into(),
            sentence_id,
        }
    }

    pub fn word(&self) -> &str {
        &self.word
    }

    pub fn sentence_id(&self) -> u32 {
        self.sentence_id
    }
}
