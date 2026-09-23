//! Заведение выбранных счётных суффиксов (issue #415): страница /counters
//! передаёт список суффиксов; создание идёт через `create_card`
//! (дедуп по content_key), пропуск — с диагностикой, не ошибкой.

use crate::domain::CounterCard;
use crate::domain::{Card, OrigaError};
use crate::traits::UserRepository;
use tracing::{info, warn};

#[derive(Clone)]
pub struct AddCounterCardsUseCase<'a, R: UserRepository> {
    repository: &'a R,
}

impl<'a, R: UserRepository> AddCounterCardsUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    /// Заводит counter-карты для перечисленных суффиксов. Возвращает
    /// `(создано, пропущено)` — пропуск = уже в колоде или суффикса нет
    /// в реестре (тихая диагностика, не ошибка операции).
    pub async fn execute(&self, suffixes: Vec<String>) -> Result<(usize, usize), OrigaError> {
        let mut user = self
            .repository
            .get_current_user()
            .await?
            .ok_or(OrigaError::CurrentUserNotExist)?;

        let mut created = 0usize;
        let mut skipped = 0usize;
        if suffixes.is_empty() {
            return Ok((0, 0));
        }
        for suffix in &suffixes {
            let Some(entry) = crate::dictionary::counters::get_counter(suffix) else {
                warn!(suffix = %suffix, "Unknown counter suffix, skipping");
                skipped += 1;
                continue;
            };
            let mut counter = CounterCard::new(entry.suffix());
            counter.ensure_registry_bindings();
            match user.create_card(Card::Counter(counter)) {
                Ok(_) => created += 1,
                Err(_) => skipped += 1,
            }
        }

        if created > 0 {
            self.repository.save(&user).await?;
        }
        info!(created, skipped, "Counter cards added");
        Ok((created, skipped))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dictionary::counters::tests::init_test_counters;
    use crate::domain::{NativeLanguage, User};
    use crate::use_cases::tests::fixtures::InMemoryUserRepository;

    #[tokio::test]
    async fn adds_selected_counters_and_skips_existing() {
        init_test_counters();
        let repo = InMemoryUserRepository::with_user(User::new(
            "t@e.st".to_string(),
            NativeLanguage::Russian,
            None,
        ));
        let (created, skipped) = AddCounterCardsUseCase::new(&repo)
            .execute(vec!["本".to_string(), "虚".to_string()])
            .await
            .unwrap();
        assert_eq!((created, skipped), (1, 1), "本 added, 虚 not in registry");

        let (created2, skipped2) = AddCounterCardsUseCase::new(&repo)
            .execute(vec!["本".to_string()])
            .await
            .unwrap();
        assert_eq!((created2, skipped2), (0, 1), "existing suffix skipped");
    }
}
