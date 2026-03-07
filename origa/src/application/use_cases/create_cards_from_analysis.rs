use super::generate_card_content::GenerateCardContentUseCase;
use crate::application::UserRepository;
use crate::domain::{Card, OrigaError, Question, StudyCard, VocabularyCard};
use tracing::{debug, info};
use ulid::Ulid;

#[derive(Debug, Clone)]
pub struct WordToCreate {
    pub base_form: String,
}

pub struct CreateCardsFromAnalysisResult {
    pub created_cards: Vec<StudyCard>,
    pub skipped_words: Vec<String>,
    pub failed_words: Vec<(String, String)>,
}

pub struct CreateCardsFromAnalysisUseCase<'a, R: UserRepository, L: crate::application::LlmService>
{
    repository: &'a R,
    generate_content_use_case: GenerateCardContentUseCase<'a, L>,
}

impl<'a, R: UserRepository, L: crate::application::LlmService>
    CreateCardsFromAnalysisUseCase<'a, R, L>
{
    pub fn new(repository: &'a R, llm_service: &'a L) -> Self {
        Self {
            repository,
            generate_content_use_case: GenerateCardContentUseCase::new(llm_service),
        }
    }

    pub async fn execute(
        &self,
        user_id: Ulid,
        words: Vec<WordToCreate>,
        set_id: Option<String>,
    ) -> Result<CreateCardsFromAnalysisResult, OrigaError> {
        debug!(user_id = %user_id, word_count = words.len(), set_id = ?set_id, "Creating cards from analysis");

        let mut user = self
            .repository
            .find_by_id(user_id)
            .await?
            .ok_or(OrigaError::UserNotFound { user_id })?;

        let mut created_cards = Vec::new();
        let mut skipped_words = Vec::new();
        let mut failed_words = Vec::new();

        for word in words {
            match self.create_card(&mut user, &word).await {
                Ok(card) => created_cards.push(card),
                Err(OrigaError::DuplicateCard { .. }) => {
                    skipped_words.push(word.base_form);
                }
                Err(e) => {
                    failed_words.push((word.base_form, e.to_string()));
                }
            }
        }

        if let Some(id) = set_id {
            user.mark_set_as_imported(id);
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
        let question = Question::new(word.base_form.clone())?;

        let content = self
            .generate_content_use_case
            .generate_content(
                &word.base_form,
                user.native_language(),
                &user.current_japanese_level(),
            )
            .await?;

        let vocabulary_card = VocabularyCard::new(question, content.answer);
        let card = Card::Vocabulary(vocabulary_card);

        user.create_card(card)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{NativeLanguage, User};
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    struct MockUserRepository {
        users: Arc<Mutex<HashMap<Ulid, User>>>,
    }

    impl MockUserRepository {
        fn new() -> Self {
            Self {
                users: Arc::new(Mutex::new(HashMap::new())),
            }
        }

        fn insert_user(&self, user: User) -> Ulid {
            let id = user.id();
            self.users.lock().unwrap().insert(id, user);
            id
        }

        fn get_user(&self, id: Ulid) -> Option<User> {
            self.users.lock().unwrap().get(&id).cloned()
        }
    }

    impl UserRepository for MockUserRepository {
        async fn find_by_id(&self, user_id: Ulid) -> Result<Option<User>, OrigaError> {
            Ok(self.users.lock().unwrap().get(&user_id).cloned())
        }

        async fn find_by_email(&self, _email: &str) -> Result<Option<User>, OrigaError> {
            Ok(None)
        }

        async fn find_by_telegram_id(
            &self,
            _telegram_id: &u64,
        ) -> Result<Option<User>, OrigaError> {
            Ok(None)
        }

        async fn save(&self, user: &User) -> Result<(), OrigaError> {
            self.users.lock().unwrap().insert(user.id(), user.clone());
            Ok(())
        }

        async fn delete(&self, _user_id: Ulid) -> Result<(), OrigaError> {
            Ok(())
        }
    }

    struct MockLlmService;

    impl crate::application::LlmService for MockLlmService {
        async fn generate_text(&self, _question: &str) -> Result<String, OrigaError> {
            Ok(r#"{"translation": "test translation"}"#.to_string())
        }
    }

    #[tokio::test]
    async fn execute_marks_set_as_imported_when_set_id_provided() {
        let repo = MockUserRepository::new();
        let user = User::new(
            "test@example.com".to_string(),
            NativeLanguage::Russian,
            None,
        );
        let user_id = user.id();
        repo.insert_user(user);

        let llm = MockLlmService;
        let use_case = CreateCardsFromAnalysisUseCase::new(&repo, &llm);

        let words = vec![WordToCreate {
            base_form: "学ぶ".to_string(),
        }];

        let result = use_case
            .execute(user_id, words, Some("test-set".to_string()))
            .await;

        assert!(result.is_ok());

        let saved_user = repo.get_user(user_id).unwrap();
        assert!(saved_user.is_set_imported("test-set"));
    }

    #[tokio::test]
    async fn execute_does_not_mark_set_when_set_id_is_none() {
        let repo = MockUserRepository::new();
        let user = User::new(
            "test@example.com".to_string(),
            NativeLanguage::Russian,
            None,
        );
        let user_id = user.id();
        repo.insert_user(user);

        let llm = MockLlmService;
        let use_case = CreateCardsFromAnalysisUseCase::new(&repo, &llm);

        let words = vec![WordToCreate {
            base_form: "学ぶ".to_string(),
        }];

        let result = use_case.execute(user_id, words, None).await;

        assert!(result.is_ok());

        let saved_user = repo.get_user(user_id).unwrap();
        assert!(!saved_user.is_set_imported("test-set"));
    }
}
