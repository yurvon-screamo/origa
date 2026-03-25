use crate::{
    domain::{tokenize_text, OrigaError, PartOfSpeech},
    traits::UserRepository,
};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzedWord {
    pub base_form: String,
    pub reading: String,
    pub part_of_speech: PartOfSpeech,
    pub is_known: bool,
    pub meaning: Option<String>,
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

    pub async fn execute(&self, text: String) -> Result<AnalyzeTextResult, OrigaError> {
        debug!(text_length = text.len(), "Analyzing text for cards");

        let user = self
            .repository
            .get_current_user()
            .await?
            .ok_or(OrigaError::CurrentUserNotExist {})?;

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

            let knowledge = user.is_word_known(&word_text);

            words.push(AnalyzedWord {
                base_form: word_text.clone(),
                reading: token.phonological_base_form().to_string(),
                part_of_speech: token.part_of_speech().clone(),
                is_known: knowledge.is_known,
                meaning: knowledge.meaning,
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
