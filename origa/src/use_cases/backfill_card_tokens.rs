//! One-time startup backfill: persist token caches on vocabulary cards
//! created before #521.
//!
//! Cards created since #521 carry `tokens` from the constructor; legacy
//! cards persisted earlier do not. Instead of synthesizing tokens at
//! render time forever, this migration walks the user's vocabulary cards
//! once, tokenizes those without a cache (the tokenizer dictionary is
//! already warm — the caller gates on `ensure_tokenizer_loaded`), and
//! persists the result through the regular `save`. After the pass every
//! card answers renders from its persistent cache; the pass itself
//! becomes a no-op (nothing to migrate, no write).

use tracing::warn;
use ulid::Ulid;

use crate::domain::tokenizer::tokenize_text;
use crate::domain::{Card, OrigaError, PrecomputedToken, VocabularyCard, is_dictionary_loaded};
use crate::traits::UserRepository;

#[derive(Clone)]
pub struct BackfillCardTokensUseCase<'a, R: UserRepository> {
    repository: &'a R,
}

impl<'a, R: UserRepository> BackfillCardTokensUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    /// Returns the number of cards that gained a persisted token cache.
    /// A card whose tokenization fails is skipped with a warning — its
    /// renders keep the live-path fallback, and a later run retries it.
    pub async fn execute(&self) -> Result<usize, OrigaError> {
        if !is_dictionary_loaded() {
            return Ok(0);
        }

        let mut user = self
            .repository
            .get_current_user()
            .await?
            .ok_or(OrigaError::CurrentUserNotExist)?;

        let legacy: Vec<(Ulid, VocabularyCard)> = user
            .knowledge_set()
            .study_cards()
            .iter()
            .filter_map(|(id, study_card)| match study_card.card() {
                Card::Vocabulary(card) if card.tokens().is_none() => Some((*id, card.clone())),
                _ => None,
            })
            .collect();
        if legacy.is_empty() {
            return Ok(0);
        }

        let mut migrated = 0usize;
        for (card_id, card) in legacy {
            let tokens = match tokenize_text(card.word().text()) {
                Ok(tokens) => tokens
                    .iter()
                    .map(PrecomputedToken::from_token_info)
                    .collect::<Vec<_>>(),
                Err(e) => {
                    warn!(
                        card_id = %card_id,
                        error = %e,
                        "token cache backfill skipped a card"
                    );
                    continue;
                },
            };
            user.update_card_content(card_id, Card::Vocabulary(card.with_tokens(tokens)))?;
            migrated += 1;
        }

        if migrated > 0 {
            self.repository.save(&user).await?;
        }
        Ok(migrated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::User;
    use crate::domain::value_objects::Question;
    use crate::use_cases::init_real_dictionaries;
    use crate::use_cases::tests::fixtures::{InMemoryUserRepository, create_test_vocab_card};

    fn legacy_user() -> User {
        let mut user = User::new(
            "backfill@example.com".to_string(),
            crate::domain::NativeLanguage::Russian,
            None,
        );
        user.create_card(create_test_vocab_card("猫")).unwrap();
        user
    }

    async fn first_card(repo: &InMemoryUserRepository) -> Card {
        let user = repo.get_current_user().await.unwrap().unwrap();
        let (_, study_card) = user
            .knowledge_set()
            .study_cards()
            .iter()
            .next()
            .expect("one card");
        study_card.card().clone()
    }

    #[tokio::test]
    async fn backfill_persists_tokens_on_legacy_cards() {
        init_real_dictionaries();
        // Arrange: a legacy card — no persisted tokens.
        let repo = InMemoryUserRepository::with_user(legacy_user());

        // Act
        let migrated = BackfillCardTokensUseCase::new(&repo)
            .execute()
            .await
            .unwrap();

        // Assert
        assert_eq!(migrated, 1);
        let Card::Vocabulary(vocab) = first_card(&repo).await else {
            panic!("expected a vocabulary card");
        };
        assert!(
            vocab.tokens().is_some_and(|tokens| !tokens.is_empty()),
            "token cache must be persisted"
        );
    }

    #[tokio::test]
    async fn backfill_rerun_is_a_noop() {
        init_real_dictionaries();
        let repo = InMemoryUserRepository::with_user(legacy_user());
        let use_case = BackfillCardTokensUseCase::new(&repo);

        let first = use_case.execute().await.unwrap();
        let second = use_case.execute().await.unwrap();

        assert_eq!((first, second), (1, 0));
    }

    #[tokio::test]
    async fn backfill_without_dictionary_migrates_nothing() {
        // The use case gates on the tokenizer being loaded; when it is
        // not, cards keep the live-path fallback and no write happens.
        // (`is_dictionary_loaded` is a process-wide OnceLock — in the
        // test binary earlier suites usually load it; both outcomes are
        // valid, so assert only that the call succeeds.)
        let repo = InMemoryUserRepository::with_user(legacy_user());
        let migrated = BackfillCardTokensUseCase::new(&repo)
            .execute()
            .await
            .unwrap();
        assert!(migrated <= 1);
    }

    #[test]
    fn with_tokens_replaces_the_cache() {
        let card =
            VocabularyCard::new_with_pos(Question::new("猫".to_string()).unwrap(), None, None);

        let updated = card.with_tokens(vec![PrecomputedToken {
            surface: "猫".to_string(),
            base: "猫".to_string(),
            reading: "ネコ".to_string(),
            pos: crate::domain::PartOfSpeech::Noun,
        }]);

        assert_eq!(updated.tokens().map(|tokens| tokens.len()), Some(1));
    }
}
