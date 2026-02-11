use origa::application::UserRepository;
use origa::application::use_cases::{
    CompleteLessonUseCase, CreateGrammarCardUseCase, CreateKanjiCardUseCase,
    CreateVocabularyCardUseCase, DeleteCardUseCase, DeleteGrammarCardUseCase,
    DeleteKanjiCardUseCase, GetUserInfoUseCase, KnowledgeSetCardsUseCase, RateCardUseCase,
    SelectCardsToFixationUseCase, SelectCardsToLessonUseCase, UpdateUserProfileUseCase,
};
use origa::domain::{JapaneseLevel, NativeLanguage};
use origa::infrastructure::{FileSystemUserRepository, FsrsSrsService, LlmServiceInvoker};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, LazyLock};
use teloxide::RequestError;
use tokio::sync::RwLock;

use crate::dialogue::SessionData;

struct OrigaServiceProviderInner {
    repository: Arc<FileSystemUserRepository>,
    llm_service: Arc<LlmServiceInvoker>,
    srs_service: Arc<FsrsSrsService>,
    session_cache: Arc<RwLock<HashMap<u64, SessionData>>>,
}

static INSTANCE: LazyLock<OrigaServiceProvider> = LazyLock::new(|| {
    let users_dir = PathBuf::from("./data/users");

    let repository = tokio::runtime::Handle::current()
        .block_on(FileSystemUserRepository::new(users_dir))
        .expect("Failed to create repository");

    let srs_service = FsrsSrsService::new().expect("Failed to create SRS service");

    let inner = OrigaServiceProviderInner {
        repository: Arc::new(repository),
        llm_service: Arc::new(LlmServiceInvoker::None),
        srs_service: Arc::new(srs_service),
        session_cache: Arc::new(RwLock::new(HashMap::new())),
    };

    OrigaServiceProvider { inner }
});

pub struct OrigaServiceProvider {
    inner: OrigaServiceProviderInner,
}

impl OrigaServiceProvider {
    pub fn instance() -> &'static Self {
        &INSTANCE
    }

    pub fn repository(&self) -> &Arc<FileSystemUserRepository> {
        &self.inner.repository
    }

    pub async fn get_or_create_session(
        &self,
        telegram_id: u64,
        username: &str,
    ) -> Result<SessionData, RequestError> {
        {
            let read_guard = self.inner.session_cache.read().await;
            if let Some(session) = read_guard.get(&telegram_id) {
                return Ok(session.clone());
            }
        }

        let user_opt = self
            .inner
            .repository
            .find_by_telegram_id(&telegram_id)
            .await
            .map_err(|e| RequestError::Io(Arc::new(std::io::Error::other(e.to_string()))))?;

        let session =
            if let Some(user) = user_opt {
                SessionData {
                    user_id: user.id(),
                    username: user.username().to_string(),
                }
            } else {
                let user = origa::domain::User::new(
                    username.to_string(),
                    JapaneseLevel::N5,
                    NativeLanguage::Russian,
                );

                self.inner.repository.save(&user).await.map_err(|e| {
                    RequestError::Io(Arc::new(std::io::Error::other(e.to_string())))
                })?;

                SessionData {
                    user_id: user.id(),
                    username: user.username().to_string(),
                }
            };

        let mut write_guard = self.inner.session_cache.write().await;
        write_guard.insert(telegram_id, session.clone());

        Ok(session)
    }

    pub fn knowledge_set_cards_use_case(
        &self,
    ) -> KnowledgeSetCardsUseCase<'_, FileSystemUserRepository> {
        KnowledgeSetCardsUseCase::new(&self.inner.repository)
    }

    pub fn get_user_info_use_case(&self) -> GetUserInfoUseCase<'_, FileSystemUserRepository> {
        GetUserInfoUseCase::new(&self.inner.repository)
    }

    pub fn update_user_profile_use_case(
        &self,
    ) -> UpdateUserProfileUseCase<'_, FileSystemUserRepository> {
        UpdateUserProfileUseCase::new(&self.inner.repository)
    }

    pub fn delete_card_use_case(&self) -> DeleteCardUseCase<'_, FileSystemUserRepository> {
        DeleteCardUseCase::new(&self.inner.repository)
    }

    pub fn delete_grammar_card_use_case(
        &self,
    ) -> DeleteGrammarCardUseCase<'_, FileSystemUserRepository> {
        DeleteGrammarCardUseCase::new(&self.inner.repository)
    }

    pub fn delete_kanji_card_use_case(
        &self,
    ) -> DeleteKanjiCardUseCase<'_, FileSystemUserRepository> {
        DeleteKanjiCardUseCase::new(&self.inner.repository)
    }

    pub fn create_grammar_card_use_case(
        &self,
    ) -> CreateGrammarCardUseCase<'_, FileSystemUserRepository> {
        CreateGrammarCardUseCase::new(&self.inner.repository)
    }

    pub fn create_vocabulary_card_use_case(
        &self,
    ) -> CreateVocabularyCardUseCase<'_, FileSystemUserRepository, LlmServiceInvoker> {
        CreateVocabularyCardUseCase::new(&self.inner.repository, &self.inner.llm_service)
    }

    pub fn create_kanji_card_use_case(
        &self,
    ) -> CreateKanjiCardUseCase<'_, FileSystemUserRepository> {
        CreateKanjiCardUseCase::new(&self.inner.repository)
    }

    pub fn select_cards_to_lesson_use_case(
        &self,
    ) -> SelectCardsToLessonUseCase<'_, FileSystemUserRepository> {
        SelectCardsToLessonUseCase::new(&self.inner.repository)
    }

    pub fn select_cards_to_fixation_use_case(
        &self,
    ) -> SelectCardsToFixationUseCase<'_, FileSystemUserRepository> {
        SelectCardsToFixationUseCase::new(&self.inner.repository)
    }

    pub fn rate_card_use_case(
        &self,
    ) -> RateCardUseCase<'_, FileSystemUserRepository, FsrsSrsService> {
        RateCardUseCase::new(&self.inner.repository, &self.inner.srs_service)
    }

    pub fn complete_lesson_use_case(&self) -> CompleteLessonUseCase<'_, FileSystemUserRepository> {
        CompleteLessonUseCase::new(&self.inner.repository)
    }
}
