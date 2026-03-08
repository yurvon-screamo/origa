use super::generate_card_content::GenerateCardContentUseCase;
use crate::domain::OrigaError;
use crate::domain::Question;
use crate::domain::tokenize_text;
use crate::domain::{Card, StudyCard, VocabularyCard};
use crate::traits::UserRepository;
use tracing::{debug, error, info, warn};
use ulid::Ulid;

#[derive(Clone)]
pub struct CreateVocabularyCardUseCase<'a, R: UserRepository> {
    repository: &'a R,
    generate_content_use_case: GenerateCardContentUseCase,
}

impl<'a, R: UserRepository> CreateVocabularyCardUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self {
            repository,
            generate_content_use_case: GenerateCardContentUseCase::new(),
        }
    }

    pub fn repository(&self) -> &'a R {
        self.repository
    }

    pub async fn execute(
        &self,
        user_id: Ulid,
        question_text: String,
    ) -> Result<Vec<StudyCard>, OrigaError> {
        let mut user = self
            .repository
            .find_by_id(user_id)
            .await?
            .ok_or(OrigaError::UserNotFound { user_id })?;

        let cards = self.create(&mut user, question_text).await?;

        self.repository.save_sync(&user).await?;

        Ok(cards)
    }

    async fn create(
        &self,
        user: &mut crate::domain::User,
        question_text: String,
    ) -> Result<Vec<StudyCard>, OrigaError> {
        let user_id = user.id();
        debug!(
            user_id = %user_id,
            question_text = %question_text,
            "Creating vocabulary card"
        );

        let mut cards = Vec::new();
        let tokens = tokenize_text(question_text.as_str())?;

        for token in tokens {
            if !token.part_of_speech().is_vocabulary_word() {
                continue;
            }

            let question_text = token.orthographic_base_form();
            let question = Question::new(question_text.to_string())?;
            let _content = self
                .generate_content_use_case
                .generate_content(question_text, user.native_language())
                .await?;

            let vocabulary_card = VocabularyCard::new(question);
            let card = Card::Vocabulary(vocabulary_card);

            let card_result = user.create_card(card);

            if let Ok(card) = card_result {
                cards.push(card);
                info!(
                    user_id = %user_id,
                    word_count = cards.len(),
                    "Vocabulary card created"
                );
            } else {
                let err = card_result.err().unwrap();
                warn!(
                    user_id = %user_id,
                    error = %err,
                    "Card already exists, skipping"
                );
                error!("Failed to create card: {}", err);
            }
        }

        Ok(cards)
    }
}
