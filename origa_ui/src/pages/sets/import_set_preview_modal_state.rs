use crate::repository::HybridUserRepository;
use crate::well_known_set::WellKnownSetLoaderImpl;
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::domain::User;
use origa::use_cases::{
    CreateCardsFromAnalysisResult, CreateCardsFromAnalysisUseCase, ImportWellKnownSetUseCase,
    WordToCreate,
};
use std::collections::HashSet;
use std::future::Future;

#[derive(Clone)]
pub struct ImportPreviewModalState {
    pub set_words: RwSignal<Vec<(String, Option<String>, bool)>>,
    pub selected_words: RwSignal<HashSet<String>>,
    pub is_loading_preview: RwSignal<bool>,
    pub is_importing: RwSignal<bool>,
    pub error_message: RwSignal<Option<String>>,
    pub current_user: RwSignal<Option<User>>,
    pub repository: HybridUserRepository,
    pub well_known_loader: WellKnownSetLoaderImpl,
    pub set_id: RwSignal<String>,
}

impl ImportPreviewModalState {
    pub fn new() -> Self {
        let current_user =
            use_context::<RwSignal<Option<User>>>().expect("current_user context not provided");
        let repository =
            use_context::<HybridUserRepository>().expect("repository context not provided");

        let well_known_loader = WellKnownSetLoaderImpl::new();

        Self {
            set_words: RwSignal::new(Vec::new()),
            selected_words: RwSignal::new(HashSet::new()),
            is_loading_preview: RwSignal::new(false),
            is_importing: RwSignal::new(false),
            error_message: RwSignal::new(None),
            current_user,
            repository,
            well_known_loader,
            set_id: RwSignal::new(String::new()),
        }
    }

    pub fn load_preview(&self, set_id: String) {
        let user_id = self
            .current_user
            .with(|u| u.as_ref().map(|u| u.id()))
            .unwrap();
        let repository = self.repository.clone();
        let well_known_loader = self.well_known_loader.clone();
        let set_words = self.set_words;
        let selected_words = self.selected_words;
        let is_loading_preview = self.is_loading_preview;
        let error = self.error_message;

        set_words.set(Vec::new());
        selected_words.set(HashSet::new());
        is_loading_preview.set(true);
        error.set(None);

        spawn_local(async move {
            let use_case = ImportWellKnownSetUseCase::new(&repository, &well_known_loader);
            match use_case.preview_set(user_id, set_id).await {
                Ok(result) => {
                    let words: Vec<(String, Option<String>, bool)> = result
                        .words
                        .iter()
                        .map(|w| (w.word.clone(), w.known_meaning.clone(), w.is_known))
                        .collect();
                    let words_to_select: HashSet<String> =
                        result.words.iter().map(|w| w.word.clone()).collect();
                    set_words.set(words);
                    selected_words.set(words_to_select);
                    is_loading_preview.set(false);
                }
                Err(e) => {
                    error.set(Some(e.to_string()));
                    is_loading_preview.set(false);
                }
            }
        });
    }

    pub fn reset(&self) {
        self.set_words.set(Vec::new());
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

    pub fn import_selected(
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
        let is_importing = self.is_importing;
        let error = self.error_message;
        let set_id = self.set_id.get();
        let set_id_opt = if set_id.is_empty() {
            None
        } else {
            Some(set_id)
        };

        async move {
            is_importing.set(true);
            error.set(None);

            let use_case = CreateCardsFromAnalysisUseCase::new(&repository);
            match use_case.execute(user_id, words_to_create, set_id_opt).await {
                Ok(result) => {
                    is_importing.set(false);
                    Ok(result)
                }
                Err(e) => {
                    error.set(Some(e.to_string()));
                    is_importing.set(false);
                    Err(e.to_string())
                }
            }
        }
    }

    pub fn set_set_id(&self, id: String) {
        self.set_id.set(id);
    }
}
