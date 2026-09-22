//! Создание counter-карт (issue #415) — единственный владелец операции.
//! Идемпотентен: дедуп по content_key (validate_unique_card ветка
//! `(Counter, Counter)` по suffix); пустой реестр — тихий `Ok(0)`.
//! Вызывается из онбординг-импорта (внутри bulk-брекета, ≤ целевого
//! уровня) — созданные карты новые и уходят в пул руки знакомства.

use crate::dictionary::counters::{counters_up_to_level, is_counters_loaded};
use crate::domain::CounterCard;
use crate::domain::{Card, JapaneseLevel, OrigaError, User};
use crate::traits::UserRepository;
use tracing::{info, warn};

#[derive(Clone)]
pub struct SeedCountersUseCase<'a, R: UserRepository> {
    repository: &'a R,
}

impl<'a, R: UserRepository> SeedCountersUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    /// Создаёт недостающие counter-карты уровней ≤ `level`.
    /// Работает по уже загруженному пользователю (внутри bulk-брекета
    /// онбординг-импорта); сохранение — ответственность вызывающего.
    pub fn seed_into_user(
        &self,
        user: &mut User,
        level: JapaneseLevel,
    ) -> Result<usize, OrigaError> {
        if !is_counters_loaded() {
            warn!("Counters registry not loaded, seeding is a no-op");
            return Ok(0);
        }
        let mut created = 0usize;
        for entry in counters_up_to_level(level) {
            let mut counter = CounterCard::new(entry.suffix());
            counter.ensure_registry_bindings();
            // create_card дедуплицирует по suffix (DuplicateCard) —
            // повторный сид для существующих карт пропускаем молча.
            if user.create_card(Card::Counter(counter)).is_ok() {
                created += 1;
            }
        }
        Ok(created)
    }

    /// Автономный путь: загрузить юзера, засидить, сохранить.
    pub async fn execute(&self, level: JapaneseLevel) -> Result<usize, OrigaError> {
        let mut user = self
            .repository
            .get_current_user()
            .await?
            .ok_or(OrigaError::CurrentUserNotExist)?;
        let created = self.seed_into_user(&mut user, level)?;
        if created > 0 {
            self.repository.save(&user).await?;
        }
        info!(created, "Counters seeded");
        Ok(created)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dictionary::counters::tests::{TEST_HON, TEST_NIN, init_test_counters};
    use crate::domain::{CardType, NativeLanguage};
    use crate::use_cases::tests::fixtures::InMemoryUserRepository;

    fn counter_suffixes(user: &User) -> Vec<String> {
        user.knowledge_set()
            .study_cards()
            .values()
            .filter_map(|sc| match sc.card() {
                Card::Counter(c) => Some(c.suffix().to_string()),
                _ => None,
            })
            .collect()
    }

    #[tokio::test]
    async fn seeding_creates_registry_counters_up_to_level() {
        init_test_counters();
        let repo = InMemoryUserRepository::new();
        let created = SeedCountersUseCase::new(&repo)
            .execute(JapaneseLevel::N5)
            .await
            .unwrap();
        assert_eq!(created, 2);

        let user = repo.get_current_user().await.unwrap().unwrap();
        let suffixes = counter_suffixes(&user);
        assert!(suffixes.contains(&TEST_HON.to_string()));
        assert!(suffixes.contains(&TEST_NIN.to_string()));
        // все карты новые — идут в пул руки знакомства
        for sc in user.knowledge_set().study_cards().values() {
            assert!(sc.memory().is_new());
            assert!(matches!(sc.card(), Card::Counter(_)));
        }
    }

    #[tokio::test]
    async fn reseeding_creates_no_duplicates() {
        init_test_counters();
        let repo = InMemoryUserRepository::new();
        let use_case = SeedCountersUseCase::new(&repo);
        use_case.execute(JapaneseLevel::N5).await.unwrap();
        let second = use_case.execute(JapaneseLevel::N5).await.unwrap();
        assert_eq!(second, 0);

        let user = repo.get_current_user().await.unwrap().unwrap();
        assert_eq!(counter_suffixes(&user).len(), 2);
    }

    #[tokio::test]
    async fn higher_level_seeding_skips_lower_priority_entries() {
        init_test_counters();
        let repo = InMemoryUserRepository::new();
        // фикстуры — N5; запрос N3 ничего не создаёт
        let created = SeedCountersUseCase::new(&repo)
            .execute(JapaneseLevel::N3)
            .await
            .unwrap();
        assert_eq!(created, 0);
        let _ = CardType::Counter;
        let _ = NativeLanguage::Russian;
    }
}
