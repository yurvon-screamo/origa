//! Мини-оценка связки счётного суффикса (issue #415): переоценивает только
//! память ячейки в режиме `CounterReview`, мимо `rate_card` — семантика
//! карты, другие связки и дневная статистика не затрагиваются. Завершение
//! композитного слота — отдельный `RecordCounterSessionUseCase`.

use crate::domain::{OrigaError, Rating};
use crate::traits::UserRepository;
use tracing::{debug, info};
use ulid::Ulid;

#[derive(Clone)]
pub struct RateCounterBindingUseCase<'a, R: UserRepository> {
    repository: &'a R,
}

impl<'a, R: UserRepository> RateCounterBindingUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        card_id: Ulid,
        number: u8,
        rating: Rating,
    ) -> Result<(), OrigaError> {
        debug!(card_id = %card_id, number, rating = ?rating, "Rating counter binding");

        let mut user = self
            .repository
            .get_current_user()
            .await?
            .ok_or(OrigaError::CurrentUserNotExist)?;

        user.rate_counter_binding(card_id, number, rating)?;

        self.repository.save(&user).await?;

        info!(card_id = %card_id, number, "Counter binding rated");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::CounterCard;
    use crate::domain::{Card, CardType, NativeLanguage, User};
    use crate::use_cases::tests::fixtures::InMemoryUserRepository;

    async fn user_with_hon(repo: &InMemoryUserRepository) -> Ulid {
        let mut user = User::new("t@e.st".to_string(), NativeLanguage::Russian, None);
        crate::dictionary::counters::tests::init_test_counters();
        let mut counter = CounterCard::new(crate::dictionary::counters::tests::TEST_HON);
        counter.ensure_registry_bindings();
        let card = user.create_card(Card::Counter(counter)).unwrap();
        repo.save_sync(&user).await.unwrap();
        *card.card_id()
    }

    #[tokio::test]
    async fn binding_rating_touches_only_that_binding_memory() {
        let repo = InMemoryUserRepository::new();
        let card_id = user_with_hon(&repo).await;
        let use_case = RateCounterBindingUseCase::new(&repo);

        use_case.execute(card_id, 3, Rating::Good).await.unwrap();

        let user = repo.get_current_user().await.unwrap().unwrap();
        match user.knowledge_set().get_card(card_id).unwrap().card() {
            Card::Counter(counter) => {
                assert!(!counter.binding_memory(3).unwrap().is_new());
                assert!(
                    counter.binding_memory(1).unwrap().is_new(),
                    "others untouched"
                );
            },
            other => panic!("expected counter card, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn binding_rating_leaves_semantic_memory_untouched() {
        let repo = InMemoryUserRepository::new();
        let card_id = user_with_hon(&repo).await;
        let use_case = RateCounterBindingUseCase::new(&repo);

        use_case.execute(card_id, 1, Rating::Good).await.unwrap();

        let user = repo.get_current_user().await.unwrap().unwrap();
        let study = user.knowledge_set().get_card(card_id).unwrap();
        assert!(
            study.memory().is_new(),
            "semantic memory stays new until the composite slot aggregate"
        );
        assert_eq!(user.knowledge_set().new_cards_studied_today(), 0);
    }

    #[tokio::test]
    async fn unknown_binding_number_is_a_domain_error() {
        let repo = InMemoryUserRepository::new();
        let card_id = user_with_hon(&repo).await;
        let use_case = RateCounterBindingUseCase::new(&repo);

        let err = use_case
            .execute(card_id, 42, Rating::Good)
            .await
            .unwrap_err();
        assert!(matches!(err, OrigaError::CounterBindingNotFound { .. }));
    }

    #[tokio::test]
    async fn non_counter_card_rejects_binding_rating() {
        let repo = InMemoryUserRepository::new();
        let mut user = User::new("t@e.st".to_string(), NativeLanguage::Russian, None);
        let vocab = user
            .create_card(Card::Vocabulary(crate::domain::VocabularyCard::new(
                crate::domain::Question::new("本".to_string()).unwrap(),
            )))
            .unwrap();
        let vocab_id = *vocab.card_id();
        repo.save_sync(&user).await.unwrap();

        let err = RateCounterBindingUseCase::new(&repo)
            .execute(vocab_id, 3, Rating::Good)
            .await
            .unwrap_err();
        assert!(matches!(err, OrigaError::CounterBindingNotFound { .. }));
        let _ = CardType::Counter; // silence unused import path in this test
    }
}
