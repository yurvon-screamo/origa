use crate::domain::{Card, OrigaError, Question, StudyCard, VocabularyCard, tokenize_text};
use crate::traits::UserRepository;
use tracing::{debug, info, warn};
use ulid::Ulid;

pub struct CreateVocabularyCardResult {
    pub created_cards: Vec<StudyCard>,
    pub skipped_no_translation: Vec<String>,
    pub skipped_duplicates: Vec<String>,
}

#[derive(Clone)]
pub struct CreateVocabularyCardUseCase<'a, R: UserRepository> {
    repository: &'a R,
}

impl<'a, R: UserRepository> CreateVocabularyCardUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    pub fn repository(&self) -> &'a R {
        self.repository
    }

    pub async fn execute(
        &self,
        user_id: Ulid,
        question_text: String,
    ) -> Result<CreateVocabularyCardResult, OrigaError> {
        let mut user = self
            .repository
            .find_by_id(user_id)
            .await?
            .ok_or(OrigaError::UserNotFound { user_id })?;

        let result = self.create(&mut user, question_text).await?;

        self.repository.save_sync(&user).await?;

        Ok(result)
    }

    async fn create(
        &self,
        user: &mut crate::domain::User,
        question_text: String,
    ) -> Result<CreateVocabularyCardResult, OrigaError> {
        let user_id = user.id();
        debug!(
            user_id = %user_id,
            question_text = %question_text,
            "Creating vocabulary card"
        );

        let mut created_cards = Vec::new();
        let mut skipped_no_translation = Vec::new();
        let mut skipped_duplicates = Vec::new();
        let tokens = tokenize_text(question_text.as_str())?;

        for token in tokens {
            if !token.part_of_speech().is_vocabulary_word() {
                continue;
            }

            let word_text = token.orthographic_base_form();

            if VocabularyCard::validate_translation(word_text, user.native_language()).is_err() {
                warn!(user_id = %user_id, word = %word_text, "Translation not found, skipping");
                skipped_no_translation.push(word_text.to_string());
                continue;
            }

            let question = Question::new(word_text.to_string())?;
            let vocabulary_card = VocabularyCard::new(question);
            let card = Card::Vocabulary(vocabulary_card);

            match user.create_card(card) {
                Ok(study_card) => {
                    created_cards.push(study_card);
                    info!(user_id = %user_id, word_count = created_cards.len(), "Vocabulary card created");
                }
                Err(OrigaError::DuplicateCard { .. }) => {
                    warn!(user_id = %user_id, word = %word_text, "Card already exists, skipping");
                    skipped_duplicates.push(word_text.to_string());
                }
                Err(e) => return Err(e),
            }
        }

        Ok(CreateVocabularyCardResult {
            created_cards,
            skipped_no_translation,
            skipped_duplicates,
        })
    }
}
