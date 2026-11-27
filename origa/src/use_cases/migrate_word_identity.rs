//! One-time cleanup of historical decks: surface-form cards and duplicates
//! are brought to the same SudachiDict lemma used at every word-creation
//! path. Called from the UI startup, next to the counters migration.

use tracing::info;

use crate::domain::tokenizer::is_dictionary_loaded;
use crate::domain::{OrigaError, WordIdentityCleanupReport};
use crate::traits::UserRepository;

#[derive(Clone)]
pub struct MigrateWordIdentityForExistingUsersUseCase<'a, R: UserRepository> {
    repository: &'a R,
}

impl<'a, R: UserRepository> MigrateWordIdentityForExistingUsersUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    pub async fn execute(&self) -> Result<WordIdentityCleanupReport, OrigaError> {
        if !is_dictionary_loaded() {
            return Ok(WordIdentityCleanupReport::default());
        }

        let Some(user) = self.repository.get_current_user().await? else {
            return Err(OrigaError::CurrentUserNotExist);
        };
        if user.is_word_identity_migrated_v1() || !user.is_onboarding_completed() {
            return Ok(WordIdentityCleanupReport::default());
        }

        // Re-read before mutation: onboarding may have finished between the
        // first read and the save. A skip here leaves the sentinel unset, so
        // the next start retries.
        let mut fresh = match self.repository.get_current_user().await? {
            Some(candidate)
                if candidate.is_onboarding_completed()
                    && !candidate.is_word_identity_migrated_v1() =>
            {
                candidate
            },
            _ => return Ok(WordIdentityCleanupReport::default()),
        };
        let language = *fresh.native_language();
        let report = fresh.knowledge_set_mut().cleanup_word_identity(&language)?;
        fresh.mark_word_identity_migrated_v1();
        self.repository.save(&fresh).await?;
        info!(
            user_id = %fresh.id(),
            rebased = report.cards_rebased,
            removed = report.cards_removed,
            "Word identity migrated for existing user"
        );
        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_objects::Question;
    use crate::domain::{Card, NativeLanguage, User, VocabularyCard};
    use crate::use_cases::tests::fixtures::{InMemoryUserRepository, real_dictionaries_or_skip};

    fn user_with_surface_cards() -> User {
        let mut user = User::new("m@e.st".to_string(), NativeLanguage::Russian, None);
        user.mark_set_as_imported(crate::domain::ONBOARDING_COMPLETED_KEY.to_string());
        for word in ["たべる", "食べる"] {
            user.create_card(Card::Vocabulary(VocabularyCard::new(
                Question::new(word.to_string()).unwrap(),
            )))
            .unwrap();
        }
        user
    }

    #[tokio::test]
    async fn migration_rebases_surface_cards_and_drops_true_duplicates() {
        real_dictionaries_or_skip!();
        // Arrange
        let repository = InMemoryUserRepository::with_user(user_with_surface_cards());

        // Act
        let report = MigrateWordIdentityForExistingUsersUseCase::new(&repository)
            .execute()
            .await
            .unwrap();

        // Assert
        assert_eq!(report.cards_removed, 1);
        let user = repository.get_current_user().await.unwrap().unwrap();
        assert_eq!(user.knowledge_set().study_cards().len(), 1);
        assert!(user.is_word_identity_migrated_v1());
        assert_eq!(
            user.knowledge_set()
                .study_cards()
                .values()
                .next()
                .unwrap()
                .card()
                .content_key(),
            "食べる"
        );
    }

    #[tokio::test]
    async fn migration_is_noop_after_sentinel_is_set() {
        real_dictionaries_or_skip!();
        // Arrange
        let repository = InMemoryUserRepository::with_user(user_with_surface_cards());
        MigrateWordIdentityForExistingUsersUseCase::new(&repository)
            .execute()
            .await
            .unwrap();

        // Act
        let report = MigrateWordIdentityForExistingUsersUseCase::new(&repository)
            .execute()
            .await
            .unwrap();

        // Assert
        assert_eq!(report.cards_rebased, 0);
        assert_eq!(report.cards_removed, 0);
    }
}
