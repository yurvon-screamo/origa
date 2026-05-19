use tracing::{debug, info};

use crate::domain::{Card, OrigaError};
use crate::traits::UserRepository;

pub struct MigrationResult {
    pub kanji_count: usize,
    pub companions_created: usize,
}

#[derive(Clone)]
pub struct MigrateKanjiCompanionsUseCase<'a, R: UserRepository> {
    repository: &'a R,
}

impl<'a, R: UserRepository> MigrateKanjiCompanionsUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    pub async fn execute(&self) -> Result<MigrationResult, OrigaError> {
        let mut user = self
            .repository
            .get_current_user()
            .await?
            .ok_or(OrigaError::CurrentUserNotExist)?;

        let kanji_chars: Vec<String> = user
            .knowledge_set()
            .study_cards()
            .iter()
            .filter_map(|(_, study_card)| {
                if let Card::Kanji(kanji_card) = study_card.card() {
                    Some(kanji_card.kanji().text().to_string())
                } else {
                    None
                }
            })
            .collect();

        info!(
            kanji_count = kanji_chars.len(),
            "Starting kanji companion migration"
        );

        let mut total_created = 0;
        for kanji_char in &kanji_chars {
            let created = user.create_companion_vocab_cards(kanji_char);
            if !created.is_empty() {
                debug!(
                    kanji = %kanji_char,
                    created = created.len(),
                    "Companion cards created during migration"
                );
            }
            total_created += created.len();
        }

        self.repository.save_sync(&user).await?;

        info!(
            kanji_count = kanji_chars.len(),
            companions_created = total_created,
            "Kanji companion migration completed"
        );

        Ok(MigrationResult {
            kanji_count: kanji_chars.len(),
            companions_created: total_created,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::{Card, KanjiCard, NativeLanguage, User};
    use crate::traits::UserRepository;
    use crate::use_cases::MigrateKanjiCompanionsUseCase;
    use crate::use_cases::tests::fixtures::{InMemoryUserRepository, init_real_dictionaries};

    fn create_user_with_kanji_cards(kanji_chars: &[&str]) -> User {
        let mut user = User::new(
            "test@example.com".to_string(),
            NativeLanguage::Russian,
            None,
        );
        for &kanji in kanji_chars {
            let card = Card::Kanji(KanjiCard::new_test(kanji.to_string()));
            user.create_card(card).expect("Failed to create kanji card");
        }
        user
    }

    #[tokio::test]
    async fn migration_creates_missing_companions() {
        init_real_dictionaries();
        let user = create_user_with_kanji_cards(&["日", "月"]);
        let repo = InMemoryUserRepository::with_user(user);
        let use_case = MigrateKanjiCompanionsUseCase::new(&repo);

        let result = use_case.execute().await.unwrap();

        assert_eq!(result.kanji_count, 2);
        assert!(
            result.companions_created > 0,
            "Expected companion cards to be created"
        );

        let user = repo.get_current_user().await.unwrap().unwrap();
        let total_cards = user.knowledge_set().study_cards().len();
        assert!(
            total_cards > 2,
            "Expected more than 2 cards (kanji + companions), got {total_cards}"
        );
    }

    #[tokio::test]
    async fn migration_is_idempotent() {
        init_real_dictionaries();
        let user = create_user_with_kanji_cards(&["日"]);
        let repo = InMemoryUserRepository::with_user(user);
        let use_case = MigrateKanjiCompanionsUseCase::new(&repo);

        let first = use_case.execute().await.unwrap();
        assert!(first.companions_created > 0);

        let second = use_case.execute().await.unwrap();
        assert_eq!(
            second.companions_created, 0,
            "Second migration should create no new companions"
        );
    }

    #[tokio::test]
    async fn migration_handles_empty_knowledge_set() {
        init_real_dictionaries();
        let user = User::new(
            "test@example.com".to_string(),
            NativeLanguage::Russian,
            None,
        );
        let repo = InMemoryUserRepository::with_user(user);
        let use_case = MigrateKanjiCompanionsUseCase::new(&repo);

        let result = use_case.execute().await.unwrap();

        assert_eq!(result.kanji_count, 0);
        assert_eq!(result.companions_created, 0);
    }
}
