use crate::{
    domain::{OrigaError, PartOfSpeech, tokenize_text},
    traits::UserRepository,
    use_cases::shared::is_word_known,
};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};
use ulid::Ulid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzedWord {
    pub text: String,
    pub base_form: String,
    pub reading: String,
    pub part_of_speech: PartOfSpeech,
    pub is_known: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub known_meaning: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzeTextResult {
    pub words: Vec<AnalyzedWord>,
    pub total_found: usize,
    pub known_count: usize,
    pub new_count: usize,
}

pub struct AnalyzeTextForCardsUseCase<'a, R: UserRepository> {
    repository: &'a R,
}

impl<'a, R: UserRepository> AnalyzeTextForCardsUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        user_id: Ulid,
        text: String,
    ) -> Result<AnalyzeTextResult, OrigaError> {
        debug!(
            user_id = %user_id,
            text_length = text.len(),
            "Analyzing text for cards"
        );

        let user = self
            .repository
            .find_by_id(user_id)
            .await?
            .ok_or(OrigaError::UserNotFound { user_id })?;

        let tokens = tokenize_text(text.as_str())?;

        debug!(token_count = tokens.len(), "Tokenized text");

        let mut words: Vec<AnalyzedWord> = Vec::new();
        let mut seen_words = std::collections::HashSet::new();

        for token in tokens {
            if !token.part_of_speech().is_vocabulary_word() {
                continue;
            }

            let word_text = token.orthographic_base_form().to_string();

            if seen_words.contains(&word_text) {
                continue;
            }
            seen_words.insert(word_text.clone());

            let (is_known, known_meaning) =
                is_word_known(&user, &word_text, user.native_language());

            words.push(AnalyzedWord {
                text: token.orthographic_surface_form().to_string(),
                base_form: word_text.clone(),
                reading: token.phonological_base_form().to_string(),
                part_of_speech: token.part_of_speech().clone(),
                is_known,
                known_meaning,
            });
        }

        let total_found = words.len();
        let known_count = words.iter().filter(|w| w.is_known).count();
        let new_count = total_found - known_count;

        info!(total_found, known_count, new_count, "Text analyzed");

        Ok(AnalyzeTextResult {
            words,
            total_found,
            known_count,
            new_count,
        })
    }
}
