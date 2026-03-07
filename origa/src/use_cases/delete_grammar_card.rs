use crate::domain::{Card, OrigaError};
use crate::traits::UserRepository;
use tracing::{debug, info};
use ulid::Ulid;

#[derive(Clone)]
pub struct DeleteGrammarCardUseCase<'a, R: UserRepository> {
    repository: &'a R,
}

impl<'a, R: UserRepository> DeleteGrammarCardUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, user_id: Ulid, rule_id: Ulid) -> Result<(), OrigaError> {
        debug!(user_id = %user_id, card_id = %rule_id, "Deleting grammar card");

        let mut user = self
            .repository
            .find_by_id(user_id)
            .await?
            .ok_or(OrigaError::UserNotFound { user_id })?;

        if let Some(card_id) = user.knowledge_set().study_cards().iter().find_map(|sc| {
            if let Card::Grammar(grammar_card) = sc.1.card()
                && grammar_card.rule_id() == &rule_id
            {
                Some(sc.0.to_owned())
            } else {
                None
            }
        }) {
            user.delete_card(card_id)?;
            info!(card_id = %card_id, "Grammar card deleted");
        } else {
            Err(OrigaError::RepositoryError {
                reason: format!("Grammar rule {} not found in knowledge set", rule_id),
            })?
        }

        self.repository.save(&user).await?;

        Ok(())
    }
}
