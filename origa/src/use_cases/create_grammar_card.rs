use crate::domain::OrigaError;
use crate::domain::{Card, GrammarRuleCard, StudyCard};
use crate::traits::UserRepository;
use tracing::{debug, info};
use ulid::Ulid;

#[derive(Clone)]
pub struct CreateGrammarCardUseCase<'a, R: UserRepository> {
    repository: &'a R,
}

impl<'a, R: UserRepository> CreateGrammarCardUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, rule_ids: Vec<Ulid>) -> Result<Vec<StudyCard>, OrigaError> {
        debug!(rule_ids = ?&rule_ids, "Creating grammar card");

        let mut user = self
            .repository
            .get_current_user()
            .await?
            .ok_or(OrigaError::CurrentUserNotExist {})?;

        let mut cards = vec![];
        for id in rule_ids {
            let card = Card::Grammar(GrammarRuleCard::new(id)?);
            let created = user.create_card(card)?;
            info!(card_id = %created.card_id(), "Grammar card created");
            cards.push(created);
        }

        self.repository.save(&user).await?;
        Ok(cards)
    }
}
