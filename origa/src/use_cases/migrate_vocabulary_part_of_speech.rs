use tracing::{debug, info, warn};

use crate::domain::{Card, OrigaError, tokenize_text};
use crate::traits::UserRepository;

pub struct PartOfSpeechMigrationResult {
    pub vocab_count: usize,
    pub migrated_count: usize,
}

#[derive(Clone)]
pub struct MigrateVocabularyPartOfSpeechUseCase<'a, R: UserRepository> {
    repository: &'a R,
}

impl<'a, R: UserRepository> MigrateVocabularyPartOfSpeechUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    pub async fn execute(&self) -> Result<PartOfSpeechMigrationResult, OrigaError> {
        let mut user = self
            .repository
            .get_current_user()
            .await?
            .ok_or(OrigaError::CurrentUserNotExist)?;

        let candidates: Vec<(ulid::Ulid, String)> = user
            .knowledge_set()
            .study_cards()
            .iter()
            .filter_map(|(id, sc)| match sc.card() {
                Card::Vocabulary(v) if v.pos().is_none() => {
                    Some((*id, v.word().text().to_string()))
                },
                _ => None,
            })
            .collect();

        let vocab_count = user
            .knowledge_set()
            .study_cards()
            .values()
            .filter(|sc| matches!(sc.card(), Card::Vocabulary(_)))
            .count();

        info!(
            vocab_count,
            candidates = candidates.len(),
            "Starting vocabulary part-of-speech migration"
        );

        let mut migrated = 0usize;
        for (card_id, word_text) in candidates {
            let pos = match tokenize_text(&word_text) {
                Ok(tokens) => match tokens.first() {
                    Some(token) => token.part_of_speech().clone(),
                    None => continue,
                },
                Err(e) => {
                    warn!(
                        card_id = %card_id,
                        word = %word_text,
                        error = %e,
                        "Tokenization failed during POS migration"
                    );
                    continue;
                },
            };

            let Some(sc) = user.knowledge_set().get_card(card_id) else {
                continue;
            };
            let Card::Vocabulary(vocab) = sc.card() else {
                continue;
            };

            let updated = vocab.clone().with_pos(pos);

            match user.update_card_content(card_id, Card::Vocabulary(updated)) {
                Ok(()) => {
                    migrated += 1;
                    debug!(card_id = %card_id, word = %word_text, "Migrated vocabulary POS");
                },
                Err(e) => {
                    warn!(
                        card_id = %card_id,
                        word = %word_text,
                        error = %e,
                        "Failed to migrate vocabulary POS"
                    );
                },
            }
        }

        // Skip persistence when nothing changed: in steady state every
        // run after the first finds zero candidates (POS already cached)
        // and `migrated` stays 0. Calling `save_sync` here would force a
        // pointless blocking write on every cold start. The caller can
        // still observe "no changes" via `migrated_count == 0` in the
        // returned `PartOfSpeechMigrationResult`. Steady-state path is
        // logged at `debug!` so it does not pollute the info stream.
        if migrated == 0 {
            debug!(
                vocab_count,
                migrated_count = 0,
                "Vocabulary part-of-speech migration skipped persistence (no changes)"
            );
            return Ok(PartOfSpeechMigrationResult {
                vocab_count,
                migrated_count: 0,
            });
        }

        self.repository.save_sync(&user).await?;

        info!(
            vocab_count,
            migrated_count = migrated,
            "Vocabulary part-of-speech migration completed"
        );

        Ok(PartOfSpeechMigrationResult {
            vocab_count,
            migrated_count: migrated,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_objects::Question;
    use crate::domain::{Card, NativeLanguage, User, VocabularyCard};
    use crate::use_cases::tests::fixtures::{InMemoryUserRepository, init_real_dictionaries};

    fn legacy_vocab_card(word: &str) -> Card {
        let json = format!(r#"{{"word":{{"text":"{word}"}},"reverse_side":null}}"#);
        let card: VocabularyCard =
            serde_json::from_str(&json).unwrap_or_else(|_| panic!("deserialize legacy {word}"));
        Card::Vocabulary(card)
    }

    fn user_with_legacy_vocab(words: &[&str]) -> User {
        let mut user = User::new(
            "test@example.com".to_string(),
            NativeLanguage::Russian,
            None,
        );
        for word in words {
            user.create_card(legacy_vocab_card(word))
                .unwrap_or_else(|_| panic!("create {word}"));
        }
        user
    }

    #[tokio::test]
    async fn migration_backfills_pos_for_legacy_vocab() {
        init_real_dictionaries();
        let user = user_with_legacy_vocab(&["猫", "食べる"]);
        let repo = InMemoryUserRepository::with_user(user);
        let use_case = MigrateVocabularyPartOfSpeechUseCase::new(&repo);

        let result = use_case.execute().await.unwrap();

        assert!(
            result.migrated_count >= 2,
            "expected POS backfill to run for both cards, got {}",
            result.migrated_count
        );

        let user = repo.get_current_user().await.unwrap().unwrap();
        for sc in user.knowledge_set().study_cards().values() {
            if let Card::Vocabulary(v) = sc.card() {
                assert!(
                    v.pos().is_some(),
                    "vocab POS should be cached after migration"
                );
            }
        }
    }

    #[tokio::test]
    async fn migration_is_idempotent() {
        init_real_dictionaries();
        let user = user_with_legacy_vocab(&["猫"]);
        let repo = InMemoryUserRepository::with_user(user);
        let use_case = MigrateVocabularyPartOfSpeechUseCase::new(&repo);

        let first = use_case.execute().await.unwrap();
        assert!(first.migrated_count > 0);

        let second = use_case.execute().await.unwrap();
        assert_eq!(
            second.migrated_count, 0,
            "second run must not re-migrate cards with cached POS"
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
        let use_case = MigrateVocabularyPartOfSpeechUseCase::new(&repo);

        let result = use_case.execute().await.unwrap();

        assert_eq!(result.vocab_count, 0);
        assert_eq!(result.migrated_count, 0);
    }

    #[test]
    fn pos_accessor_returns_none_for_legacy_card() {
        let card: VocabularyCard =
            serde_json::from_str(r#"{"word":{"text":"猫"},"reverse_side":null}"#)
                .expect("deserialize legacy");
        assert!(card.pos().is_none());
    }

    #[test]
    fn with_pos_sets_cache() {
        let card = VocabularyCard::new(Question::new("猫".to_string()).unwrap())
            .with_pos(crate::domain::PartOfSpeech::Noun);
        assert_eq!(card.pos(), Some(crate::domain::PartOfSpeech::Noun));
    }
}
