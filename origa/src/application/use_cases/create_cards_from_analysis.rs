use super::generate_card_content::GenerateCardContentUseCase;
use crate::application::UserRepository;
use crate::domain::{Card, OrigaError, Question, StudyCard, VocabularyCard};
use tracing::{debug, info};
use ulid::Ulid;

#[derive(Debug, Clone)]
pub struct WordToCreate {
    pub base_form: String,
}

pub struct CreateCardsFromAnalysisResult {
    pub created_cards: Vec<StudyCard>,
    pub skipped_words: Vec<String>,
    pub failed_words: Vec<(String, String)>,
}

pub struct CreateCardsFromAnalysisUseCase<'a, R: UserRepository, L: crate::application::LlmService>
{
    repository: &'a R,
    generate_content_use_case: GenerateCardContentUseCase<'a, L>,
}

impl<'a, R: UserRepository, L: crate::application::LlmService>
    CreateCardsFromAnalysisUseCase<'a, R, L>
{
    pub fn new(repository: &'a R, llm_service: &'a L) -> Self {
        Self {
            repository,
            generate_content_use_case: GenerateCardContentUseCase::new(llm_service),
        }
    }

    pub async fn execute(
        &self,
        user_id: Ulid,
        words: Vec<WordToCreate>,
    ) -> Result<CreateCardsFromAnalysisResult, OrigaError> {
        debug!(user_id = %user_id, word_count = words.len(), "Creating cards from analysis");

        let mut user = self
            .repository
            .find_by_id(user_id)
            .await?
            .ok_or(OrigaError::UserNotFound { user_id })?;

        let mut created_cards = Vec::new();
        let mut skipped_words = Vec::new();
        let mut failed_words = Vec::new();

        for word in words {
            match self.create_card(&mut user, &word).await {
                Ok(card) => created_cards.push(card),
                Err(OrigaError::DuplicateCard { .. }) => {
                    skipped_words.push(word.base_form);
                }
                Err(e) => {
                    failed_words.push((word.base_form, e.to_string()));
                }
            }
        }

        self.repository.save_sync(&user).await?;

        info!(
            created_count = created_cards.len(),
            skipped_count = skipped_words.len(),
            failed_count = failed_words.len(),
            "Cards from analysis created"
        );

        Ok(CreateCardsFromAnalysisResult {
            created_cards,
            skipped_words,
            failed_words,
        })
    }

    async fn create_card(
        &self,
        user: &mut crate::domain::User,
        word: &WordToCreate,
    ) -> Result<StudyCard, OrigaError> {
        let question = Question::new(word.base_form.clone())?;

        let content = self
            .generate_content_use_case
            .generate_content(
                &word.base_form,
                user.native_language(),
                &user.current_japanese_level(),
            )
            .await?;

        let vocabulary_card = VocabularyCard::new(question, content.answer);
        let card = Card::Vocabulary(vocabulary_card);

        user.create_card(card)
    }
}
