use crate::services::grammar_service::GrammarService;
use crate::services::kanji_service::KanjiService;
use crate::services::study_service::StudyService;
use crate::services::user_service::UserService;
use crate::services::vocabulary_service::VocabularyService;
use leptos::prelude::*;

#[derive(Clone)]
pub struct AppServices {
    pub kanji_service: KanjiService,
    pub vocabulary_service: VocabularyService,
    pub grammar_service: GrammarService,
    pub study_service: StudyService,
    pub user_service: UserService,
}

impl AppServices {
    pub fn new() -> Self {
        Self {
            kanji_service: KanjiService::new(),
            vocabulary_service: VocabularyService::new(),
            grammar_service: GrammarService::new(),
            study_service: StudyService::new(),
            user_service: UserService::new(),
        }
    }
}

#[component]
pub fn ServicesProvider(services: AppServices, children: Children) -> impl IntoView {
    provide_context(services.clone());
    provide_context(services.kanji_service);
    provide_context(services.vocabulary_service);
    provide_context(services.grammar_service);
    provide_context(services.study_service);
    provide_context(services.user_service);
    children()
}
