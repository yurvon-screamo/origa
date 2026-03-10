use crate::domain::OrigaError;
use crate::traits::{UserRepository, WellKnownSetLoader};
use crate::use_cases::{CreateVocabularyCardUseCase, shared::is_word_known};
use tracing::{debug, info};
use ulid::Ulid;

pub struct ImportWellKnownSetResult {
    pub total_created_count: usize,
    pub skipped_words: Vec<String>,
    pub errors: Vec<String>,
}

pub struct SetPreviewWord {
    pub word: String,
    pub known_meaning: Option<String>,
    pub is_known: bool,
}

pub struct SetPreviewResult {
    pub words: Vec<SetPreviewWord>,
    pub total_count: usize,
    pub known_count: usize,
}

pub struct ImportWellKnownSetUseCase<'a, R: UserRepository, W: WellKnownSetLoader> {
    create_card_use_case: CreateVocabularyCardUseCase<'a, R>,
    loader: &'a W,
}

impl<'a, R: UserRepository, W: WellKnownSetLoader> ImportWellKnownSetUseCase<'a, R, W> {
    pub fn new(repository: &'a R, loader: &'a W) -> Self {
        Self {
            create_card_use_case: CreateVocabularyCardUseCase::new(repository),
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
            Ok(result) => {
                for word in words {
                    if !result
                        .created_cards
                        .iter()
                        .any(|c| c.card().content_key() == *word)
                    {
                        skipped_words.push(word.clone());
                    }
                }
                created_count += result.created_cards.len();
            }
            Err(e) => {
                tracing::error!("Failed to create cards for words {}: {}", question, e);
                errors.push(e.to_string());
            }
        }

        Ok((created_count, skipped_words, errors))
    }

    pub async fn preview_set(
        &self,
        user_id: Ulid,
        set_id: String,
    ) -> Result<SetPreviewResult, OrigaError> {
        debug!(user_id = %user_id, set_id = %set_id, "Previewing well-known set");

        let user = self
            .create_card_use_case
            .repository()
            .find_by_id(user_id)
            .await?
            .ok_or(OrigaError::UserNotFound { user_id })?;

        let set = self.loader.load_set(set_id.clone()).await?;
        let words = set.words();

        let mut preview_words = Vec::new();
        let mut known_count = 0;

        for word in words {
            let (is_known, known_meaning) = is_word_known(&user, word, user.native_language());
            if is_known {
                known_count += 1;
            }
            preview_words.push(SetPreviewWord {
                word: word.clone(),
                known_meaning,
                is_known,
            });
        }

        let total_count = preview_words.len();

        info!(total_count, known_count, "Well-known set preview completed");

        Ok(SetPreviewResult {
            words: preview_words,
            total_count,
            known_count,
        })
    }
}
