use crate::application::{
    CreateVocabularyCardUseCase, LlmService, UserRepository, WellKnownSetLoader,
};
use crate::domain::OrigaError;
use tracing::{debug, info};
use ulid::Ulid;

pub struct ImportWellKnownSetResult {
    pub total_created_count: usize,
    pub skipped_words: Vec<String>,
    pub errors: Vec<String>,
}

pub struct ImportWellKnownSetUseCase<'a, R: UserRepository, L: LlmService, W: WellKnownSetLoader> {
    create_card_use_case: CreateVocabularyCardUseCase<'a, R, L>,
    loader: &'a W,
}

impl<'a, R: UserRepository, L: LlmService, W: WellKnownSetLoader>
    ImportWellKnownSetUseCase<'a, R, L, W>
{
    pub fn new(repository: &'a R, llm_service: &'a L, loader: &'a W) -> Self {
        Self {
            create_card_use_case: CreateVocabularyCardUseCase::new(repository, llm_service),
            loader,
        }
    }

    pub async fn execute(
        &self,
        user_id: Ulid,
        set_id: String,
    ) -> Result<ImportWellKnownSetResult, OrigaError> {
        debug!(user_id = %user_id, set_id = %set_id, "Importing well-known set");

        let mut total_created_count = 0;
        let mut total_skipped_words = Vec::new();
        let mut total_errors = Vec::new();

        let set = self.loader.load_set(set_id.clone()).await?;
        info!(word_count = set.words().len(), "Well-known set loaded");

        let (created, skipped, errors) = self.process_words(user_id, set.words()).await?;

        total_created_count += created;
        total_skipped_words.extend(skipped);
        total_errors.extend(errors);

        info!(
            total_created_count = total_created_count,
            skipped_count = total_skipped_words.len(),
            errors_count = total_errors.len(),
            "Well-known set import completed"
        );

        Ok(ImportWellKnownSetResult {
            total_created_count,
            skipped_words: total_skipped_words,
            errors: total_errors,
        })
    }

    async fn process_words(
        &self,
        user_id: Ulid,
        words: &[String],
    ) -> Result<(usize, Vec<String>, Vec<String>), OrigaError> {
        let mut created_count = 0;
        let mut skipped_words = Vec::new();
        let mut errors = Vec::new();

        let question = words.join(";");

        match self
            .create_card_use_case
            .execute(user_id, question.clone())
            .await
        {
            Ok(cards) => {
                for word in words {
                    if !cards.iter().any(|c| c.card().question().text() == word) {
                        skipped_words.push(word.clone());
                    }
                }
                created_count += cards.len();
            }
            Err(e) => {
                tracing::error!("Failed to create cards for words {}: {}", question, e);
                errors.push(e.to_string());
            }
        }

        Ok((created_count, skipped_words, errors))
    }
}
