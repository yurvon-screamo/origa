//! Завершение композитного слота счётного суффикса (issue #415): UI вызывает
//! ровно один раз после последнего мини-вопроса пачки. Агрегатный рейтинг
//! применяется к памяти СЕМАНТИКИ карты в режиме `StandardLesson` штатным
//! рейтинговым путём: серия без ошибок → Good, любая ошибка → Again.
//!
//! Это одновременно:
//! - live/recalc-согласованность — `last_rating`/`last_review_date` семантики
//!   обновлены, `recalculate_daily_stats` фолдит ту же запись;
//! - механизм сцепления — ошибки на связках укорачивают интервал семантики,
//!   и карта возвращается к невыученным ячейкам раньше;
//! - бюджет новичков — `was_new` инкремент происходит ровно один раз
//!   (мини-оценки идут мимо рейтингового пути).

use crate::domain::{OrigaError, RateMode, Rating, RatingContext};
use crate::traits::UserRepository;
use tracing::{debug, info};
use ulid::Ulid;

#[derive(Clone)]
pub struct RecordCounterSessionUseCase<'a, R: UserRepository> {
    repository: &'a R,
}

impl<'a, R: UserRepository> RecordCounterSessionUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    /// `all_correct`: каждая мини-связка пачки отвечена верно.
    pub async fn execute(&self, card_id: Ulid, all_correct: bool) -> Result<(), OrigaError> {
        let rating = if all_correct {
            Rating::Good
        } else {
            Rating::Again
        };
        debug!(card_id = %card_id, rating = ?rating, "Recording counter session aggregate");

        let mut user = self
            .repository
            .get_current_user()
            .await?
            .ok_or(OrigaError::CurrentUserNotExist)?;

        // Штатный путь: память семантики в StandardLesson (Counter без
        // ремапа — см. KnowledgeSet::rate_card), дневная запись одна,
        // `was_new` учтён автоматически.
        user.rate_card(
            card_id,
            rating,
            RateMode::StandardLesson,
            RatingContext::Explicit,
        )?;

        self.repository.save(&user).await?;

        info!(card_id = %card_id, "Counter session recorded");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dictionary::counters::tests::init_test_counters;
    use crate::domain::CounterCard;
    use crate::domain::{Card, NativeLanguage, User};
    use crate::use_cases::RateCounterBindingUseCase;
    use crate::use_cases::tests::fixtures::InMemoryUserRepository;

    fn seeded_hon_user() -> User {
        init_test_counters();
        let mut user = User::new("t@e.st".to_string(), NativeLanguage::Russian, None);
        let mut counter = CounterCard::new(crate::dictionary::counters::tests::TEST_HON);
        counter.ensure_registry_bindings();
        user.create_card(Card::Counter(counter)).unwrap();
        user
    }

    #[tokio::test]
    async fn session_aggregate_moves_semantic_memory_and_spends_one_new_budget() {
        let repo = InMemoryUserRepository::new();
        let user = seeded_hon_user();
        let card_id = *user
            .knowledge_set()
            .study_cards()
            .values()
            .next()
            .unwrap()
            .card_id();
        repo.save_sync(&user).await.unwrap();

        // Мини-оценки связок — мимо рейтингового пути.
        let binding = RateCounterBindingUseCase::new(&repo);
        binding.execute(card_id, 1, Rating::Good).await.unwrap();
        binding.execute(card_id, 3, Rating::Good).await.unwrap();
        let user = repo.get_current_user().await.unwrap().unwrap();
        assert_eq!(user.knowledge_set().new_cards_studied_today(), 0);

        RecordCounterSessionUseCase::new(&repo)
            .execute(card_id, true)
            .await
            .unwrap();

        let user = repo.get_current_user().await.unwrap().unwrap();
        let study = user.knowledge_set().get_card(card_id).unwrap();
        assert!(
            !study.memory().is_new(),
            "aggregate moves the card out of the new pool"
        );
        assert_eq!(
            user.knowledge_set().new_cards_studied_today(),
            1,
            "exactly one unit of the newcomer budget per composite slot"
        );
    }

    #[tokio::test]
    async fn any_binding_failure_aggregates_to_again() {
        let repo = InMemoryUserRepository::new();
        let user = seeded_hon_user();
        let card_id = *user
            .knowledge_set()
            .study_cards()
            .values()
            .next()
            .unwrap()
            .card_id();
        repo.save_sync(&user).await.unwrap();

        RecordCounterSessionUseCase::new(&repo)
            .execute(card_id, false)
            .await
            .unwrap();

        let user = repo.get_current_user().await.unwrap().unwrap();
        let study = user.knowledge_set().get_card(card_id).unwrap();
        assert!(!study.memory().is_new());
        // Короткий Again-интервал: карта вернётся к невыученным связкам.
        assert!(study.memory().next_review_date().is_some());
    }

    #[tokio::test]
    async fn second_session_does_not_double_the_new_budget() {
        let repo = InMemoryUserRepository::new();
        let user = seeded_hon_user();
        let card_id = *user
            .knowledge_set()
            .study_cards()
            .values()
            .next()
            .unwrap()
            .card_id();
        repo.save_sync(&user).await.unwrap();

        let session = RecordCounterSessionUseCase::new(&repo);
        session.execute(card_id, true).await.unwrap();
        session.execute(card_id, true).await.unwrap();

        let user = repo.get_current_user().await.unwrap().unwrap();
        assert_eq!(user.knowledge_set().new_cards_studied_today(), 1);
    }
}
