use crate::application::{CreateVocabularyCardUseCase, LlmService, UserRepository};
use crate::domain::OrigaError;
use crate::domain::{WellKnownSets, load_well_known_set};
use ulid::Ulid;

pub struct ImportWellKnownSetResult {
    pub total_created_count: usize,
    pub skipped_words: Vec<String>,
}

pub struct ImportWellKnownSetUseCase<'a, R: UserRepository, L: LlmService> {
    create_card_use_case: CreateVocabularyCardUseCase<'a, R, L>,
}

impl<'a, R: UserRepository, L: LlmService> ImportWellKnownSetUseCase<'a, R, L> {
    pub fn new(repository: &'a R, llm_service: &'a L) -> Self {
        Self {
            create_card_use_case: CreateVocabularyCardUseCase::new(repository, llm_service),
        }
    }

    pub async fn execute(
        &self,
        user_id: Ulid,
        set: WellKnownSets,
    ) -> Result<ImportWellKnownSetResult, OrigaError> {
        let mut total_created_count = 0;
        let mut total_skipped_words = Vec::new();

        let words = load_well_known_set(&set)?;

        let (created, skipped) = self.process_words(user_id, words.words()).await?;

        total_created_count += created;
        total_skipped_words.extend(skipped);

        Ok(ImportWellKnownSetResult {
            total_created_count,
            skipped_words: total_skipped_words,
        })
    }

    async fn process_words(
        &self,
        user_id: Ulid,
        words: &[String],
    ) -> Result<(usize, Vec<String>), OrigaError> {
        let mut created_count = 0;
        let mut skipped_words = Vec::new();

        for word in words {
            let question = word.clone();

            match self
                .create_card_use_case
                .execute(user_id, question.clone())
                .await
            {
                Ok(_) => {
                    created_count += 1;
                }
                Err(OrigaError::DuplicateCard { .. }) => {
                    skipped_words.push(question);
                }
                Err(e) => {
                    tracing::error!("Failed to create card for word {}: {}", question, e);
                    skipped_words.push(question);
                }
            }
        }

        Ok((created_count, skipped_words))
    }
}
