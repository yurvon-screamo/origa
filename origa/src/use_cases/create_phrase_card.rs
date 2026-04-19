use crate::domain::OrigaError;
use crate::domain::{Card, PhraseCard, StudyCard};
use crate::traits::UserRepository;
use tracing::{debug, info};
use ulid::Ulid;

#[derive(Clone)]
pub struct CreatePhraseCardUseCase<'a, R: UserRepository> {
    repository: &'a R,
}

impl<'a, R: UserRepository> CreatePhraseCardUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, phrase_ids: Vec<Ulid>) -> Result<Vec<StudyCard>, OrigaError> {
        debug!(phrase_ids = ?&phrase_ids, "Creating phrase cards");

        let mut user = self
            .repository
            .get_current_user()
            .await?
            .ok_or(OrigaError::CurrentUserNotExist {})?;

        let mut cards = vec![];
        for id in phrase_ids {
            let card = Card::Phrase(PhraseCard::new(id)?);
            let created = user.create_card(card)?;
            info!(card_id = %created.card_id(), "Phrase card created");
            cards.push(created);
        }

        self.repository.save(&user).await?;
        Ok(cards)
    }
}
