use crate::application::UserRepository;
use crate::domain::OrigaError;
use crate::domain::get_rule_by_id;
use crate::domain::{Card, GrammarRuleCard, StudyCard};
use ulid::Ulid;

#[derive(Clone)]
pub struct CreateGrammarCardUseCase<'a, R: UserRepository> {
    repository: &'a R,
}

impl<'a, R: UserRepository> CreateGrammarCardUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        user_id: Ulid,
        rule_ids: Vec<Ulid>,
    ) -> Result<Vec<StudyCard>, OrigaError> {
        let mut user = self
            .repository
            .find_by_id(user_id)
            .await?
            .ok_or(OrigaError::UserNotFound { user_id })?;

        let mut cards = vec![];
        for id in rule_ids {
            let rule = get_rule_by_id(&id).ok_or_else(|| OrigaError::RepositoryError {
                reason: format!("Grammar rule {} not found", id),
            })?;
            let card = Card::Grammar(GrammarRuleCard::new(rule, user.native_language())?);
            let created = user.create_card(card)?;
            cards.push(created);
        }

        self.repository.save(&user).await?;
        Ok(cards)
    }
}
