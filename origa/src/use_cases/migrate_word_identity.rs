//! Read-only duplicate scan for existing decks (uwuwu#1463).
//!
//! Word creation already collapses script and inflection variants through
//! `from_text`. Re-tokenizing a stored word is not equivalent and was shown to
//! rename independent dictionary articles, so this use case never mutates or
//! saves the user record — it reports exact-key duplicates for inspection.

use tracing::info;

use crate::domain::{OrigaError, WordIdentityScanReport};
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
        let report = crate::domain::scan_duplicate_cards(user.knowledge_set());
        if report.duplicate_cards > 0 {
            info!(
                groups = report.groups_scanned,
                duplicates = report.duplicate_cards,
                "Duplicate cards found in deck"
            );
        }
        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_objects::Question;
    use crate::domain::{Card, NativeLanguage, User, VocabularyCard};
    use crate::use_cases::tests::fixtures::InMemoryUserRepository;

    fn user_with_cards() -> User {
        let mut user = User::new("m@e.st".to_string(), NativeLanguage::Russian, None);
        for word in ["食べる", "見える"] {
            user.create_card(Card::Vocabulary(VocabularyCard::new(
                Question::new(word.to_string()).unwrap(),
            )))
            .unwrap();
        }
        user
    }

    #[tokio::test]
    async fn scan_reports_duplicates_without_changing_the_deck() {
        // Arrange
        let repository = InMemoryUserRepository::with_user(user_with_cards());
        let before = repository
            .get_current_user()
            .await
            .unwrap()
            .unwrap()
            .knowledge_set()
            .study_cards()
            .len();

        // Act
        let report = ScanWordIdentityUseCase::new(&repository)
            .execute()
            .await
            .unwrap();

        // Assert
        assert_eq!(report.groups_scanned, 0);
        assert_eq!(report.duplicate_cards, 0);
        let after = repository
            .get_current_user()
            .await
            .unwrap()
            .unwrap()
            .knowledge_set()
            .study_cards()
            .len();
        assert_eq!(after, before);
    }
}
