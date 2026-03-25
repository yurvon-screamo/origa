use crate::domain::{Card, OrigaError, StudyCard, VocabularyCard};
use crate::traits::UserRepository;
use tracing::{debug, info, warn};

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
        question_text: String,
    ) -> Result<CreateVocabularyCardResult, OrigaError> {
        let mut user = self
            .repository
            .get_current_user()
            .await?
            .ok_or(OrigaError::CurrentUserNotExist {})?;

        let result = self.create(&mut user, question_text).await?;

        self.repository.save_sync(&user).await?;

        Ok(result)
    }

    async fn create(
        &self,
        user: &mut crate::domain::User,
        question_text: String,
    ) -> Result<CreateVocabularyCardResult, OrigaError> {
        debug!(
            user_id = %user.id(),
            question_text = %question_text,
            "Creating vocabulary card"
        );

        let result = VocabularyCard::from_text(&question_text, user.native_language());

        for word in &result.skipped_no_translation {
            warn!(user_id = %user.id(), word = %word, "Translation not found, skipping");
        }

        let mut created_cards = Vec::new();
        let mut skipped_duplicates = Vec::new();

        for vocab_card in result.cards {
            let card = Card::Vocabulary(vocab_card);
            match user.create_card(card) {
                Ok(study_card) => {
                    created_cards.push(study_card);
                },
                Err(OrigaError::DuplicateCard { question }) => {
                    warn!(user_id = %user.id(), word = %question, "Card already exists, skipping");
                    skipped_duplicates.push(question);
                },
                Err(e) => return Err(e),
            }
        }

        info!(
            user_id = %user.id(),
            created_count = created_cards.len(),
            "Vocabulary cards created"
        );

        Ok(CreateVocabularyCardResult {
            created_cards,
            skipped_no_translation: result.skipped_no_translation,
            skipped_duplicates,
        })
    }
}
