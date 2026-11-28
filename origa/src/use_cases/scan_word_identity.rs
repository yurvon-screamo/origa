//! Read-only duplicate scan for existing decks (uwuwu#1463).
//!
//! Word creation already collapses script and inflection variants through
//! `from_text`. Re-tokenizing a stored word is not equivalent and was shown to
//! rename independent dictionary articles, so this use case never mutates or
//! saves the user record — it reports exact-key duplicates for inspection.

use crate::domain::{OrigaError, WordIdentityScanReport, scan_duplicate_cards};
use crate::traits::UserRepository;

#[derive(Clone)]
pub struct ScanWordIdentityUseCase<'a, R: UserRepository> {
    repository: &'a R,
}

impl<'a, R: UserRepository> ScanWordIdentityUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    pub async fn execute(&self) -> Result<WordIdentityScanReport, OrigaError> {
        let Some(user) = self.repository.get_current_user().await? else {
            return Ok(WordIdentityScanReport::default());
        };
        Ok(scan_duplicate_cards(user.knowledge_set()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::future::Future;
    use ulid::Ulid;

    use crate::domain::value_objects::Question;
    use crate::domain::{Card, NativeLanguage, User, VocabularyCard};
    use crate::use_cases::tests::fixtures::InMemoryUserRepository;

    struct CountingRepository {
        inner: InMemoryUserRepository,
        saves: Cell<usize>,
    }

    impl UserRepository for CountingRepository {
        fn get_current_user(&self) -> impl Future<Output = Result<Option<User>, OrigaError>> {
            self.inner.get_current_user()
        }

        fn save(&self, user: &User) -> impl Future<Output = Result<(), OrigaError>> {
            self.saves.set(self.saves.get() + 1);
            self.inner.save(user)
        }

        fn delete(&self, user_id: Ulid) -> impl Future<Output = Result<(), OrigaError>> {
            self.inner.delete(user_id)
        }
    }

    fn user_with_duplicate() -> User {
        let mut user = User::new("m@e.st".to_string(), NativeLanguage::Russian, None);
        let card = Card::Vocabulary(VocabularyCard::new(
            Question::new("食べる".to_string()).unwrap(),
        ));
        user.create_card(card.clone()).unwrap();
        // A duplicate can exist in historical data even though create_card
        // rejects it today; insert it directly to scan that shape.
        user.knowledge_set_mut()
            .study_cards_mut_for_test()
            .insert(Ulid::new(), crate::domain::StudyCard::new(card));
        user
    }

    #[tokio::test]
    async fn scan_reports_duplicates_and_never_saves_the_user() {
        // Arrange
        let repository = CountingRepository {
            inner: InMemoryUserRepository::with_user(user_with_duplicate()),
            saves: Cell::new(0),
        };

        // Act
        let report = ScanWordIdentityUseCase::new(&repository)
            .execute()
            .await
            .unwrap();

        // Assert
        assert_eq!(report.group_count(), 1);
        assert_eq!(report.duplicate_cards(), 1);
        assert_eq!(report.groups[0].word, "食べる");
        assert_eq!(repository.saves.get(), 0);
    }
}
