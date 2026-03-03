use crate::repository::HybridUserRepository;
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::application::{
    AnalyzeTextForCardsUseCase, AnalyzedWord, CreateCardsFromAnalysisUseCase, CreateCardsFromAnalysisResult, WordToCreate,
};
use origa::domain::User;
use origa::infrastructure::LlmServiceInvoker;
use std::collections::HashSet;

#[derive(Clone)]
pub struct PreviewModalState {
    pub input_text: RwSignal<String>,
    pub analyzed_words: RwSignal<Vec<AnalyzedWord>>,
    pub selected_words: RwSignal<HashSet<String>>,
    pub is_analyzing: RwSignal<bool>,
    pub is_creating: RwSignal<bool>,
    pub error_message: RwSignal<Option<String>>,
    pub current_user: RwSignal<Option<User>>,
    pub repository: HybridUserRepository,
    pub llm_service: LlmServiceInvoker,
}

impl PreviewModalState {
    pub fn new(is_open: RwSignal<bool>) -> Self {
        let current_user =
            use_context::<RwSignal<Option<User>>>().expect("current_user context not provided");
        let repository =
            use_context::<HybridUserRepository>().expect("repository context not provided");
        let llm_service =
            use_context::<LlmServiceInvoker>().expect("llm_service context not provided");

        let selected_words = RwSignal::new(HashSet::new());

        Effect::new({
            let selected_words_clone = selected_words;
            move |_| {
                if is_open.get() {
                    selected_words_clone.set(HashSet::new());
                }
            }
        });

        Self {
            input_text: RwSignal::new(String::new()),
            analyzed_words: RwSignal::new(Vec::new()),
            selected_words,
            is_analyzing: RwSignal::new(false),
            is_creating: RwSignal::new(false),
            error_message: RwSignal::new(None),
            current_user,
            repository,
            llm_service,
        }
    }

    pub fn analyze_text(&self) {
        let user_id = self
            .current_user
            .with(|u| u.as_ref().map(|u| u.id()))
            .unwrap();
        let text = self.input_text.get();
        let repository = self.repository.clone();
        let analyzed_words = self.analyzed_words;
        let selected_words = self.selected_words;
        let is_analyzing = self.is_analyzing;
        let error = self.error_message;

        is_analyzing.set(true);
        error.set(None);

        spawn_local(async move {
            let use_case = AnalyzeTextForCardsUseCase::new(&repository);
            match use_case.execute(user_id, text).await {
                Ok(result) => {
                    let words_to_select: HashSet<String> =
                        result.words.iter().map(|w| w.base_form.clone()).collect();
                    analyzed_words.set(result.words);
                    selected_words.set(words_to_select);
                    is_analyzing.set(false);
                }
                Err(e) => {
                    error.set(Some(e.to_string()));
                    is_analyzing.set(false);
                }
            }
        });
    }

    pub fn reset(&self) {
        self.input_text.set(String::new());
        self.analyzed_words.set(Vec::new());
        self.selected_words.set(HashSet::new());
        self.error_message.set(None);
    }

    pub fn toggle_word(&self, word: String) {
        self.selected_words.update(|selected| {
            if selected.contains(&word) {
                selected.remove(&word);
            } else {
                selected.insert(word);
            }
        });
    }

    pub fn create_cards(
        &self,
    ) -> impl Future<Output = Result<CreateCardsFromAnalysisResult, String>> {
        let user_id = self
            .current_user
            .with(|u| u.as_ref().map(|u| u.id()))
            .unwrap();
        let selected_words = self.selected_words.get();
        let words_to_create: Vec<WordToCreate> = selected_words
            .into_iter()
            .map(|base_form| WordToCreate { base_form })
            .collect();
        let repository = self.repository.clone();
        let llm_service = self.llm_service.clone();
        let is_creating = self.is_creating;
        let error = self.error_message;

        async move {
            is_creating.set(true);
            error.set(None);

            let use_case = CreateCardsFromAnalysisUseCase::new(&repository, &llm_service);
            match use_case.execute(user_id, words_to_create).await {
                Ok(result) => {
                    is_creating.set(false);
                    Ok(result)
                }
                Err(e) => {
                    error.set(Some(e.to_string()));
                    is_creating.set(false);
                    Err(e.to_string())
                }
            }
        }
    }
}
