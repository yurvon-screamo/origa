use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::dictionary::phrase::get_phrase_by_id;
use crate::domain::value_objects::{NativeLanguage, Question};
use crate::domain::{OrigaError, value_objects::Answer};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PhraseCard {
    phrase_id: Ulid,
}

impl PhraseCard {
    pub fn new(phrase_id: Ulid) -> Result<Self, OrigaError> {
        get_phrase_by_id(&phrase_id).ok_or(OrigaError::PhraseNotFound { phrase_id })?;
        Ok(Self { phrase_id })
    }

    pub fn phrase_id(&self) -> &Ulid {
        &self.phrase_id
    }

    pub fn question(&self) -> Result<Question, OrigaError> {
        let entry = get_phrase_by_id(&self.phrase_id).ok_or(OrigaError::PhraseNotFound {
            phrase_id: self.phrase_id,
        })?;
        Question::new(entry.text().to_string()).map_err(|e| OrigaError::InvalidQuestion {
            reason: e.to_string(),
        })
    }

    pub fn answer(&self, lang: &NativeLanguage) -> Result<Answer, OrigaError> {
        let entry = get_phrase_by_id(&self.phrase_id).ok_or(OrigaError::PhraseNotFound {
            phrase_id: self.phrase_id,
        })?;

        if let Some(translation) = entry.translation(lang) {
            if !translation.is_empty() {
                return Answer::new(translation.to_string()).map_err(|e| {
                    OrigaError::InvalidAnswer {
                        reason: e.to_string(),
                    }
                });
            }
        }

        Answer::new(entry.text().to_string()).map_err(|e| OrigaError::InvalidAnswer {
            reason: e.to_string(),
        })
    }

    pub fn audio_file(&self) -> Result<String, OrigaError> {
        let entry = get_phrase_by_id(&self.phrase_id).ok_or(OrigaError::PhraseNotFound {
            phrase_id: self.phrase_id,
        })?;
        Ok(entry.audio_file().to_string())
    }

    pub fn tokens(&self) -> Result<Vec<String>, OrigaError> {
        let entry = get_phrase_by_id(&self.phrase_id).ok_or(OrigaError::PhraseNotFound {
            phrase_id: self.phrase_id,
        })?;
        Ok(entry.tokens().to_vec())
    }

    #[cfg(test)]
    pub fn new_test_with_id(phrase_id: Ulid) -> Self {
        Self { phrase_id }
    }
}
