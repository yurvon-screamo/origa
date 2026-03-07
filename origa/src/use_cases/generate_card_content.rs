use crate::domain::OrigaError;
use crate::domain::get_translation;
use crate::domain::{Answer, NativeLanguage};
use tracing::{debug, info};

#[derive(Clone, Default)]
pub struct GenerateCardContentUseCase {}

#[derive(Debug, Clone, PartialEq)]
pub struct CardContent {
    pub answer: Answer,
}

impl GenerateCardContentUseCase {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn generate_content(
        &self,
        question_text: &str,
        native_language: &NativeLanguage,
    ) -> Result<CardContent, OrigaError> {
        debug!(question_text, "Generating card content");

        if let Some(result) = self.try_get_from_dictionary(question_text, native_language)? {
            info!(
                question_text,
                source = "dictionary",
                "Card content generated"
            );
            return Ok(result);
        }

        Err(OrigaError::DictionaryNotFound {
            reason: format!("Dictionary not found for word '{}'", question_text),
        })
    }

    fn try_get_from_dictionary(
        &self,
        question_text: &str,
        native_language: &NativeLanguage,
    ) -> Result<Option<CardContent>, OrigaError> {
        debug!(question_text, "Checking dictionary for word");

        if let Some(translation) = get_translation(question_text, native_language) {
            let answer = Answer::new(translation)?;
            return Ok(Some(CardContent { answer }));
        }

        Ok(None)
    }
}
