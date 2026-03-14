use crate::domain::{Card, OrigaError, Question, StudyCard, VocabularyCard};
use crate::traits::UserRepository;
use tracing::{debug, info, warn};

#[derive(Debug, Clone)]
pub struct WordToCreate {
    pub base_form: String,
}

pub struct CreateCardsFromAnalysisResult {
    pub created_cards: Vec<StudyCard>,
    pub skipped_words: Vec<String>,
    pub failed_words: Vec<(String, String)>,
}

pub struct CreateCardsFromAnalysisUseCase<'a, R: UserRepository> {
    repository: &'a R,
}

impl<'a, R: UserRepository> CreateCardsFromAnalysisUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        words: Vec<WordToCreate>,
        set_id: Option<String>,
    ) -> Result<CreateCardsFromAnalysisResult, OrigaError> {
        debug!(word_count = words.len(), set_id = ?set_id, "Creating cards from analysis");

        let mut user = self
            .repository
            .get_current_user()
            .await?
            .ok_or(OrigaError::CurrentUserNotExist {})?;

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

        if let Some(id) = set_id {
            user.mark_set_as_imported(id);
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
        VocabularyCard::validate_translation(&word.base_form, user.native_language()).inspect_err(
            |e| {
                warn!(word = %word.base_form, error = %e, "Translation not found");
            },
        )?;

        let question = Question::new(word.base_form.clone())?;
        let vocabulary_card = VocabularyCard::new(question);
        let card = Card::Vocabulary(vocabulary_card);

        user.create_card(card)
    }
}
