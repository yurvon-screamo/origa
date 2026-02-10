use crate::application::user_repository::UserRepository;
use crate::domain::{Card, OrigaError};
use ulid::Ulid;

#[derive(Clone)]
pub struct DeleteKanjiCardUseCase<'a, R: UserRepository> {
    repository: &'a R,
}

impl<'a, R: UserRepository> DeleteKanjiCardUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, user_id: Ulid, kanji: String) -> Result<(), OrigaError> {
        let mut user = self
            .repository
            .find_by_id(user_id)
            .await?
            .ok_or(OrigaError::UserNotFound { user_id })?;

        if let Some(card_id) = user.knowledge_set().study_cards().iter().find_map(|sc| {
            if let Card::Kanji(kanji_card) = sc.1.card()
                && kanji_card.kanji().text() == kanji
            {
                Some(sc.0.to_owned())
            } else {
                None
            }
        }) {
            user.delete_card(card_id)?;
        } else {
            Err(OrigaError::RepositoryError {
                reason: format!("Kanji rule {} not found in knowledge set", kanji),
            })?
        }

        self.repository.save(&user).await?;

        Ok(())
    }
}
