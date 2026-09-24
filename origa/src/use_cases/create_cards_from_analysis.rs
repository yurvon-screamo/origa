use crate::domain::{Card, OrigaError, StudyCard, VocabularyCard};
use crate::traits::UserRepository;
use tracing::{debug, info, warn};

#[derive(Debug, Clone)]
pub struct WordToCreate {
    pub base_form: String,
    /// POS из анализа текста: `Suffix` + суффикс в реестре счётчиков →
    /// создаётся counter-карта, а не словарная (issue #415). `None` —
    /// пути без анализа (импорт сетов): прежнее поведение.
    pub part_of_speech: Option<crate::domain::PartOfSpeech>,
}

impl WordToCreate {
    pub fn word(base_form: impl Into<String>) -> Self {
        Self {
            base_form: base_form.into(),
            part_of_speech: None,
        }
    }
}

pub struct CreateCardsFromAnalysisResult {
    pub created_cards: Vec<StudyCard>,
    pub skipped_words: Vec<String>,
    pub failed_words: Vec<(String, String)>,
}

pub struct CreateCardsFromAnalysisUseCase<'a, R: UserRepository> {
    repository: &'a R,
}

impl<'a, R: UserRepository> CreateCardsFromAnalysisUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    /// Creates cards from a list of words with optional set import marking.
    ///
    /// - `words` — list of words to create cards from
    /// - `set_ids` — IDs of sets to mark as imported (None = no marking)
    pub async fn execute(
        &self,
        words: Vec<WordToCreate>,
        set_ids: Option<Vec<String>>,
    ) -> Result<CreateCardsFromAnalysisResult, OrigaError> {
        debug!(word_count = words.len(), set_ids = ?set_ids, "Creating cards from analysis");

        let mut user = self
            .repository
            .get_current_user()
            .await?
            .ok_or(OrigaError::CurrentUserNotExist)?;

        let mut created_cards = Vec::new();
        let mut skipped_words = Vec::new();
        let mut failed_words = Vec::new();

        for word in words {
            match self.create_card(&mut user, &word).await {
                Ok(card) => created_cards.push(card),
                Err(OrigaError::DuplicateCard { .. }) => {
                    skipped_words.push(word.base_form);
                },
                Err(e) => {
                    failed_words.push((word.base_form, e.to_string()));
                },
            }
        }

        if let Some(ids) = set_ids {
            user.mark_sets_as_imported(ids);
        }

        self.repository.save_sync(&user).await?;

        info!(
            created_count = created_cards.len(),
            skipped_count = skipped_words.len(),
            failed_count = failed_words.len(),
            "Cards from analysis created"
        );

        Ok(CreateCardsFromAnalysisResult {
            created_cards,
            skipped_words,
            failed_words,
        })
    }

    async fn create_card(
        &self,
        user: &mut crate::domain::User,
        word: &WordToCreate,
    ) -> Result<StudyCard, OrigaError> {
        // Счётный суффикс из текста (POS Suffix из анализа + реестр):
        // словарная карта не нужна — контент живёт в реестре.
        if word.part_of_speech == Some(crate::domain::PartOfSpeech::Suffix)
            && crate::dictionary::counters::get_counter(&word.base_form).is_some()
        {
            let mut counter = crate::domain::CounterCard::new(&word.base_form);
            counter.ensure_registry_bindings();
            return user.create_card(Card::Counter(counter));
        }

        let result = VocabularyCard::from_text(&word.base_form, user.native_language());

        for skipped in &result.skipped_no_translation {
            warn!(word = %skipped, "Translation not found");
        }

        let vocab_card =
            result
                .cards
                .into_iter()
                .next()
                .ok_or_else(|| OrigaError::VocabularyNotFound {
                    word: word.base_form.clone(),
                })?;

        let card = Card::Vocabulary(vocab_card);
        user.create_card(card)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Card, NativeLanguage, PartOfSpeech};
    use crate::use_cases::tests::fixtures::InMemoryUserRepository;

    fn user() -> crate::domain::User {
        crate::domain::User::new("t@e.st".to_string(), NativeLanguage::Russian, None)
    }

    /// POS Suffix из анализа + суффикс в реестре → counter-карта
    /// (контент из реестра, все связки на месте).
    #[tokio::test]
    async fn suffix_word_creates_a_counter_card() {
        crate::dictionary::counters::tests::init_test_counters();
        let repo = InMemoryUserRepository::with_user(user());
        let result = CreateCardsFromAnalysisUseCase::new(&repo)
            .execute(
                vec![WordToCreate {
                    base_form: "本".to_string(),
                    part_of_speech: Some(PartOfSpeech::Suffix),
                }],
                None,
            )
            .await
            .unwrap();
        assert_eq!(result.created_cards.len(), 1);
        match result.created_cards[0].card() {
            Card::Counter(counter) => {
                assert_eq!(counter.suffix(), "本");
                assert!(!counter.bindings().is_empty());
            },
            other => panic!("expected a counter card, got {other:?}"),
        }
    }

    /// POS Noun (standalone 本 — «книга») → обычная словарная карта:
    /// омонимы не превращаются в счётчики без контекста из текста.
    #[tokio::test]
    async fn noun_homonym_stays_a_vocabulary_card() {
        crate::dictionary::counters::tests::init_test_counters();
        let repo = InMemoryUserRepository::with_user(user());
        let result = CreateCardsFromAnalysisUseCase::new(&repo)
            .execute(
                vec![WordToCreate {
                    base_form: "本".to_string(),
                    part_of_speech: Some(PartOfSpeech::Noun),
                }],
                None,
            )
            .await
            .unwrap();
        assert!(matches!(
            result.created_cards[0].card(),
            Card::Vocabulary(_)
        ));
    }
}
