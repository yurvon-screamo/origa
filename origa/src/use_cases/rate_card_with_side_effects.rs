use crate::domain::{Card, OrigaError, RateMode, Rating};
use crate::traits::UserRepository;
use crate::use_cases::{CreateGrammarCardUseCase, RateCardUseCase};
use tracing::warn;
use ulid::Ulid;

pub struct RateCardWithSideEffectsUseCase<'a, R: UserRepository> {
    repository: &'a R,
}

impl<'a, R: UserRepository> RateCardWithSideEffectsUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        card_id: Ulid,
        rate_mode: RateMode,
        rating: Rating,
        grammar_rule_id: Option<Ulid>,
    ) -> Result<(), OrigaError> {
        RateCardUseCase::new(self.repository)
            .execute(card_id, rate_mode, rating)
            .await?;

        if let Some(grammar_rule_id) = grammar_rule_id {
            self.handle_grammar_dual_rating(grammar_rule_id, rating)
                .await;
        }

        Ok(())
    }

    async fn handle_grammar_dual_rating(&self, grammar_rule_id: Ulid, rating: Rating) {
        let Some(user) = self.repository.get_current_user().await.ok().flatten() else {
            return;
        };

        let existing_card_id = user
            .knowledge_set()
            .study_cards()
            .iter()
            .find(|(_, study_card)| {
                let Card::Grammar(grammar_card) = study_card.card() else {
                    return false;
                };
                grammar_card.rule_id() == &grammar_rule_id
            })
            .map(|(id, _)| *id);

        if let Some(card_id) = existing_card_id {
            if let Err(e) = RateCardUseCase::new(self.repository)
                .execute(card_id, RateMode::GrammarReview, rating)
                .await
            {
                warn!(error = ?e, "Failed to rate grammar card during dual rating");
            }
            return;
        }

        let Ok(new_cards) = CreateGrammarCardUseCase::new(self.repository)
            .execute(vec![grammar_rule_id])
            .await
        else {
            return;
        };

        let Some(first_card) = new_cards.first() else {
            return;
        };

        if let Err(e) = RateCardUseCase::new(self.repository)
            .execute(*first_card.card_id(), RateMode::GrammarReview, rating)
            .await
        {
            warn!(error = ?e, "Failed to rate newly created grammar card during dual rating");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Card, GrammarRuleCard, NativeLanguage, Question, User, VocabularyCard};
    use crate::use_cases::tests::fixtures::InMemoryUserRepository;

    fn create_test_user_with_vocab() -> User {
        User::new(
            "test@example.com".to_string(),
            NativeLanguage::Russian,
            None,
        )
    }

    fn create_vocab_card(word: &str) -> Card {
        Card::Vocabulary(VocabularyCard::new(
            Question::new(word.to_string()).unwrap(),
        ))
    }

    #[tokio::test]
    async fn rates_card_without_grammar_dual_rating() {
        let mut user = create_test_user_with_vocab();
        let study_card = user.create_card(create_vocab_card("猫")).unwrap();
        let card_id = *study_card.card_id();
        let repo = InMemoryUserRepository::with_user(user);

        let use_case = RateCardWithSideEffectsUseCase::new(&repo);

        let result = use_case
            .execute(card_id, RateMode::StandardLesson, Rating::Good, None)
            .await;

        assert!(result.is_ok());

        let updated_user = repo.get_current_user().await.unwrap().unwrap();
        let rated_card = updated_user
            .knowledge_set()
            .study_cards()
            .get(&card_id)
            .unwrap();
        assert!(!rated_card.is_new());
    }

    #[tokio::test]
    async fn rates_card_with_grammar_dual_rating_creates_new_grammar_card() {
        let mut user = create_test_user_with_vocab();
        let study_card = user.create_card(create_vocab_card("猫")).unwrap();
        let card_id = *study_card.card_id();
        let repo = InMemoryUserRepository::with_user(user.clone());

        let grammar_rule_id = Ulid::new();
        let use_case = RateCardWithSideEffectsUseCase::new(&repo);

        let result = use_case
            .execute(
                card_id,
                RateMode::StandardLesson,
                Rating::Good,
                Some(grammar_rule_id),
            )
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn rates_card_with_existing_grammar_card() {
        let mut user = create_test_user_with_vocab();
        let study_card = user.create_card(create_vocab_card("猫")).unwrap();
        let card_id = *study_card.card_id();

        let grammar_rule_id = Ulid::new();
        let grammar_card = GrammarRuleCard::new_test_with_id(grammar_rule_id);
        let grammar_study = user.create_card(Card::Grammar(grammar_card)).unwrap();
        let grammar_card_id = *grammar_study.card_id();

        let repo = InMemoryUserRepository::with_user(user);
        let use_case = RateCardWithSideEffectsUseCase::new(&repo);

        let result = use_case
            .execute(
                card_id,
                RateMode::StandardLesson,
                Rating::Good,
                Some(grammar_rule_id),
            )
            .await;

        assert!(result.is_ok());

        let updated_user = repo.get_current_user().await.unwrap().unwrap();
        let rated_grammar = updated_user
            .knowledge_set()
            .study_cards()
            .get(&grammar_card_id)
            .unwrap();
        assert!(!rated_grammar.is_new());
    }

    #[tokio::test]
    async fn returns_error_for_nonexistent_card() {
        let user = create_test_user_with_vocab();
        let repo = InMemoryUserRepository::with_user(user);

        let use_case = RateCardWithSideEffectsUseCase::new(&repo);

        let result = use_case
            .execute(Ulid::new(), RateMode::StandardLesson, Rating::Good, None)
            .await;

        assert!(result.is_err());
    }
}
