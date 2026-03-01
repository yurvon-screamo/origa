use crate::application::UserRepository;
use crate::domain::{OrigaError, PartOfSpeech, tokenize_text};
use serde::{Deserialize, Serialize};
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
        let user = self
            .repository
            .find_by_id(user_id)
            .await?
            .ok_or(OrigaError::UserNotFound { user_id })?;

        let tokens = tokenize_text(text.as_str())?;

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

            let (is_known, known_meaning) = self.check_if_known(&user, &word_text);

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

        Ok(AnalyzeTextResult {
            words,
            total_found,
            known_count,
            new_count,
        })
    }

    fn check_if_known(&self, user: &crate::domain::User, word: &str) -> (bool, Option<String>) {
        for study_card in user.knowledge_set().study_cards().values() {
            if let crate::domain::Card::Vocabulary(vocab_card) = study_card.card()
                && vocab_card.word().text() == word
            {
                return (true, Some(vocab_card.meaning().text().to_string()));
            }
        }
        (false, None)
    }
}
