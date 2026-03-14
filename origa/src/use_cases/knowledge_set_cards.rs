use crate::domain::OrigaError;
use crate::domain::StudyCard;
use crate::traits::UserRepository;
use tracing::{debug, info};

#[derive(Clone)]
pub struct KnowledgeSetCardsUseCase<'a, R: UserRepository> {
    repository: &'a R,
}

impl<'a, R: UserRepository> KnowledgeSetCardsUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    pub async fn execute(&self) -> Result<Vec<StudyCard>, OrigaError> {
        debug!("Getting knowledge set cards");

        let user = self
            .repository
            .get_current_user()
            .await?
            .ok_or(OrigaError::CurrentUserNotExist {})?;

        let cards: Vec<StudyCard> = user
            .knowledge_set()
            .study_cards()
            .values()
            .cloned()
            .collect();

        info!(count = cards.len(), "Knowledge set cards retrieved");

        Ok(cards)
    }
}
