//! Миграция счётных суффиксов для существующих юзеров (issue #415, §8):
//! вызывается лоадером на каждом старте сразу после загрузки реестра —
//! прецедент `MigrateGrammarCardsUseCase` (идемпотентный повтор — no-op).
//!
//! Охват: ТОЛЬКО детект из вокаба (surface-первичен: числительное рядом
//! с суффиксом — 三日 → 日; 日本 ↛ 本; бонус — пара `Numeral → Suffix` в
//! многотокенных `card.tokens`). Полное множество ≤ уровня приходит
//! онбординг-импортом: миграция не засыпает counter-картами юзера, чей
//! вокаб суффиксов не встречал (иначе пустые колоды и empty-state
//! перестают существовать). Все создаваемые карты новые — пул руки
//! знакомства; дневной лимит тратится только закрытием руки.

use std::collections::HashSet;

use crate::dictionary::counters::{
    counters_up_to_level, is_counters_loaded, suffix_detected_in_word,
};
use crate::domain::CounterCard;
use crate::domain::{Card, OrigaError, PartOfSpeech};
use crate::traits::UserRepository;
use tracing::info;

#[derive(Clone)]
pub struct MigrateCountersForExistingUsersUseCase<'a, R: UserRepository> {
    repository: &'a R,
}

/// Прекомпьют-бонус детекта: карта несёт пару соседних токенов
/// `Numeral → Suffix` (from_known_word-карты; from_text одно-токенные —
/// потому surface-детект и первичен).
fn tokens_detect_counter(sc: &crate::domain::StudyCard, suffix: &str) -> bool {
    let Card::Vocabulary(vocab) = sc.card() else {
        return false;
    };
    let Some(tokens) = vocab.tokens() else {
        return false;
    };
    let mut prev_numeral = false;
    for token in tokens {
        if prev_numeral && token.surface == suffix && token.pos == PartOfSpeech::Suffix {
            return true;
        }
        prev_numeral = token.pos == PartOfSpeech::Numeral;
    }
    false
}

impl<'a, R: UserRepository> MigrateCountersForExistingUsersUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    pub async fn execute(&self) -> Result<usize, OrigaError> {
        if !is_counters_loaded() {
            return Ok(0);
        }

        let mut user = self
            .repository
            .get_current_user()
            .await?
            .ok_or(OrigaError::CurrentUserNotExist)?;

        // Разовая накатка (решение владельца): прогнав миграцию один раз,
        // больше не сканируем вокаб на каждом старте — новые счётчики
        // заводятся кандидатом анализа текста сразу при добавлении слов.
        if user.is_counters_migrated_v1() {
            return Ok(0);
        }

        // Юзер в онбординге исключён ГОНКОЙ: стартовая миграция читает
        // юзера до скипа и сейвит копию без флага завершения — скип
        // затирался, ProtectedRoute гонял юзера по кругу на /onboarding
        // (красный CI e2e, run 35892085689). Sentinel НЕ ставим: прогон
        // повторится на старте после завершения онбординга.
        if !user.is_onboarding_completed() {
            return Ok(0);
        }

        // Пред-проход: суффиксы существующих counter-карт. После первого
        // прогона миграция — дешёвый no-op без create_card-попыток.
        let existing: HashSet<String> = user
            .knowledge_set()
            .study_cards()
            .values()
            .filter_map(|sc| match sc.card() {
                Card::Counter(counter) => Some(counter.suffix().to_string()),
                _ => None,
            })
            .collect();

        // Слова юзера — материал детекта (persisted card.tokens читаем
        // напрямую: порядок install_user_card_precompute относительно
        // загрузки counters.json лоадерами не гарантирован).
        let words: Vec<crate::domain::StudyCard> = user
            .knowledge_set()
            .study_cards()
            .values()
            .filter(|sc| matches!(sc.card(), Card::Vocabulary(_)))
            .cloned()
            .collect();

        let level = user.current_japanese_level();
        let mut created = 0usize;
        let mut detected_suffixes: Vec<&str> = Vec::new();
        for entry in counters_up_to_level(level) {
            let suffix = entry.suffix();
            if existing.contains(suffix) {
                continue;
            }
            let detected = words.iter().any(|sc| {
                let word = match sc.card() {
                    Card::Vocabulary(v) => v.word().text(),
                    _ => "",
                };
                suffix_detected_in_word(word, suffix) || tokens_detect_counter(sc, suffix)
            });
            if !detected {
                continue;
            }
            detected_suffixes.push(suffix);
            let mut counter = CounterCard::new(suffix);
            counter.ensure_registry_bindings();
            if user.create_card(Card::Counter(counter)).is_ok() {
                created += 1;
            }
        }
        if !detected_suffixes.is_empty() {
            info!(
                detected = detected_suffixes.join(","),
                "counters met in the user's vocabulary — migrated"
            );
        }

        // Sentinel пишем и при нулевом охвате: «накатили» — факт свершившийся.
        user.mark_counters_migrated_v1();
        self.repository.save(&user).await?;
        info!(user_id = %user.id(), created, "Counters migrated for existing user");
        Ok(created)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dictionary::counters::tests::{TEST_HON, TEST_NIN, init_test_counters};
    use crate::domain::CounterCard;
    use crate::domain::User;
    use crate::domain::value_objects::Question;
    use crate::domain::{JapaneseLevel, NativeLanguage, VocabularyCard};
    use crate::use_cases::tests::fixtures::InMemoryUserRepository;

    fn user_with_words(words: &[&str]) -> crate::domain::User {
        let mut user = User::new("m@e.st".to_string(), NativeLanguage::Russian, None);
        // Миграция гейтится завершённым онбордингом — фикстура игрока
        // «после онбординга».
        user.mark_set_as_imported(crate::domain::ONBOARDING_COMPLETED_KEY.to_string());
        for w in words {
            user.create_card(Card::Vocabulary(VocabularyCard::new(
                Question::new(w.to_string()).unwrap(),
            )))
            .unwrap();
        }
        user
    }

    #[tokio::test]
    async fn migration_creates_only_detected_suffixes() {
        init_test_counters();
        // Охват — только детект: 一本 приносит 本; 日本 — нет; 人/日 не
        // задетекчены — не создаются (полное ≤ уровня приходит
        // онбординг-импортом).
        let repo = InMemoryUserRepository::with_user(user_with_words(&["一本", "日本"]));
        let created = MigrateCountersForExistingUsersUseCase::new(&repo)
            .execute()
            .await
            .unwrap();
        assert_eq!(created, 1);
        let user = repo.get_current_user().await.unwrap().unwrap();
        let suffixes: Vec<String> = user
            .knowledge_set()
            .study_cards()
            .values()
            .filter_map(|sc| match sc.card() {
                Card::Counter(c) => Some(c.suffix().to_string()),
                _ => None,
            })
            .collect();
        assert_eq!(suffixes, vec![TEST_HON.to_string()]);
    }

    /// Разовость: sentinel ставится первым прогоном (даже с нулевым
    /// охватом) и гасит все последующие.
    #[tokio::test]
    async fn migration_runs_exactly_once_per_user() {
        init_test_counters();
        let repo = InMemoryUserRepository::with_user(user_with_words(&[]));
        let use_case = MigrateCountersForExistingUsersUseCase::new(&repo);
        // Пустой вокаб: 0 создано, но sentinel записан…
        assert_eq!(use_case.execute().await.unwrap(), 0);
        // …слово-связка появляется ПОСЛЕ накатки — повтор не заводит:
        let mut user = repo.get_current_user().await.unwrap().unwrap();
        user.create_card(Card::Vocabulary(VocabularyCard::new(
            crate::domain::value_objects::Question::new("一本".to_string()).unwrap(),
        )))
        .unwrap();
        repo.save(&user).await.unwrap();
        assert_eq!(
            use_case.execute().await.unwrap(),
            0,
            "sentinel suppresses reruns"
        );
    }

    /// Юзер в онбординге не мигрируется и НЕ получает sentinel: стартовая
    /// миграция сейвила бы копию юзера до скипа — гонка затирала флаг
    /// завершения онбординга (красный e2e CI).
    #[tokio::test]
    async fn migration_waits_for_onboarding_completion() {
        init_test_counters();
        let mut user = User::new("m@e.st".to_string(), NativeLanguage::Russian, None);
        user.create_card(Card::Vocabulary(VocabularyCard::new(
            Question::new("一本".to_string()).unwrap(),
        )))
        .unwrap();
        let repo = InMemoryUserRepository::with_user(user);
        let use_case = MigrateCountersForExistingUsersUseCase::new(&repo);

        assert_eq!(
            use_case.execute().await.unwrap(),
            0,
            "onboarding user is skipped"
        );
        let stored = repo.get_current_user().await.unwrap().unwrap();
        assert!(
            !stored.is_counters_migrated_v1(),
            "no sentinel until onboarding completes"
        );
    }

    #[tokio::test]
    async fn migration_is_a_cheap_noop_on_rerun() {
        init_test_counters();
        let repo = InMemoryUserRepository::with_user(user_with_words(&["一本"]));
        let use_case = MigrateCountersForExistingUsersUseCase::new(&repo);
        // Только детект: 一本 → 本, далее no-op.
        assert_eq!(use_case.execute().await.unwrap(), 1);
        assert_eq!(use_case.execute().await.unwrap(), 0);
        assert_eq!(use_case.execute().await.unwrap(), 0);
    }

    #[tokio::test]
    async fn migration_creates_cards_as_new_for_the_acquaintance_pool() {
        init_test_counters();
        let repo = InMemoryUserRepository::with_user(user_with_words(&["二本"]));
        MigrateCountersForExistingUsersUseCase::new(&repo)
            .execute()
            .await
            .unwrap();
        let user = repo.get_current_user().await.unwrap().unwrap();
        for sc in user.knowledge_set().study_cards().values() {
            if matches!(sc.card(), Card::Counter(_)) {
                assert!(sc.memory().is_new());
            }
        }
        assert_eq!(user.knowledge_set().new_cards_studied_today(), 0);
    }

    #[tokio::test]
    async fn migration_skips_counters_the_user_already_has() {
        init_test_counters();
        let mut user = user_with_words(&["一本"]);
        let mut hon = CounterCard::new(TEST_HON);
        hon.ensure_registry_bindings();
        user.create_card(Card::Counter(hon)).unwrap();
        let mut nin = CounterCard::new(TEST_NIN);
        nin.ensure_registry_bindings();
        user.create_card(Card::Counter(nin)).unwrap();
        let repo = InMemoryUserRepository::with_user(user);

        let created = MigrateCountersForExistingUsersUseCase::new(&repo)
            .execute()
            .await
            .unwrap();
        // 本 и 人 уже есть; детект по 一本 не пересоздаёт существующие.
        assert_eq!(created, 0, "existing counters are never recreated");
        let _ = JapaneseLevel::N5;
    }
}
