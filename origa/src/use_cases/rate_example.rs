//! #528 v3: rate (or lazily create) the SRS card of an example sentence.
//!
//! One unified entry for BOTH showings of an example:
//! - a premiere companion (the card does not exist yet): create it and
//!   apply the first review DIRECTLY — a premiere is not a review in any
//!   axis (not counted in the lesson progress, no daily-history marks,
//!   no ghost ladder), so it deliberately bypasses `rate_card`;
//! - a due review (the card exists): the normal `rate_card` pipeline
//!   with `RateMode::PhraseReview` (retention 0.70 / 365d — «like
//!   phrases», the owner decision) and `RatingContext::Explicit` so the
//!   ghost ladder advances.
//!
//! The word owner's own schedule is never touched.

use crate::domain::{Card, ExampleCard, OrigaError, RateMode, Rating, RatingContext, User};
use crate::traits::UserRepository;
use tracing::info;
use ulid::Ulid;

#[derive(Clone)]
pub struct RateExampleUseCase<'a, R: UserRepository> {
    repository: &'a R,
}

impl<'a, R: UserRepository> RateExampleUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    /// `understood = false` → Again, `true` → Good (the binary
    /// self-assessment of the example card).
    pub async fn execute(
        &self,
        word: &str,
        sentence_id: u32,
        understood: bool,
    ) -> Result<(), OrigaError> {
        let rating = if understood {
            Rating::Good
        } else {
            Rating::Again
        };
        let mut user = self
            .repository
            .get_current_user()
            .await?
            .ok_or(OrigaError::CurrentUserNotExist)?;

        match find_example_card(&user, word, sentence_id) {
            Some(card_id) => {
                // Due review: the full pipeline (history, ghost ladder).
                user.rate_card(
                    card_id,
                    rating,
                    RateMode::PhraseReview,
                    RatingContext::Explicit,
                )?;
            },
            None => {
                // Premiere: create + first review directly — NOT via
                // rate_card (a premiere is not a review in any axis).
                let created =
                    user.create_card(Card::Example(ExampleCard::new(word, sentence_id)))?;
                user.apply_example_premiere(*created.card_id(), rating)?;
                info!(
                    word,
                    sentence_id,
                    ?rating,
                    "Example SRS card created (premiere)"
                );
            },
        }

        self.repository.save(&user).await?;
        Ok(())
    }
}

fn find_example_card(user: &User, word: &str, sentence_id: u32) -> Option<Ulid> {
    user.knowledge_set()
        .study_cards()
        .iter()
        .find(|(_, sc)| match sc.card() {
            Card::Example(ec) => ec.word() == word && ec.sentence_id() == sentence_id,
            _ => false,
        })
        .map(|(id, _)| *id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_objects::NativeLanguage;
    use crate::use_cases::tests::fixtures::InMemoryUserRepository;

    fn repo_with_user() -> InMemoryUserRepository {
        InMemoryUserRepository::with_user(User::new(
            "e2e@example.com".to_string(),
            NativeLanguage::Russian,
            None,
        ))
    }

    async fn example_cards_count(repo: &InMemoryUserRepository, word: &str, sid: u32) -> usize {
        repo.get_current_user()
            .await
            .unwrap()
            .unwrap()
            .knowledge_set()
            .study_cards()
            .values()
            .filter(|sc| matches!(sc.card(), Card::Example(ec) if ec.word() == word && ec.sentence_id() == sid))
            .count()
    }

    #[tokio::test]
    async fn premiere_creates_the_card_with_the_rating_applied() {
        let repo = repo_with_user();
        RateExampleUseCase::new(&repo)
            .execute("たべる", 7, false)
            .await
            .expect("premiere rate");

        let user = repo.get_current_user().await.unwrap().unwrap();
        let sc = user
            .knowledge_set()
            .study_cards()
            .values()
            .find(|sc| {
                matches!(sc.card(), Card::Example(ec) if ec.word() == "たべる" && ec.sentence_id() == 7)
            })
            .expect("card created");
        assert!(!sc.memory().is_new(), "Again applied on creation — not new");
    }

    #[tokio::test]
    async fn due_path_rates_the_existing_card_no_duplicates() {
        let repo = repo_with_user();
        let use_case = RateExampleUseCase::new(&repo);
        use_case
            .execute("たべる", 7, false)
            .await
            .expect("premiere");
        use_case.execute("たべる", 7, true).await.expect("due");

        assert_eq!(
            example_cards_count(&repo, "たべる", 7).await,
            1,
            "upsert — no duplicates"
        );
    }
}
