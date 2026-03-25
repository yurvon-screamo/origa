use crate::loaders::WellKnownSetLoaderImpl;
use crate::pages::sets::types::PreviewWord;
use crate::repository::HybridUserRepository;
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::traits::{UserRepository, WellKnownSetLoader};
use origa::use_cases::{
    CreateCardsFromAnalysisResult, CreateCardsFromAnalysisUseCase, WordToCreate,
};
use std::collections::HashMap;
use std::collections::HashSet;
use std::future::Future;

#[derive(Clone)]
pub struct ImportPreviewModalState {
    pub preview_words: RwSignal<Vec<PreviewWord>>,
    pub selected_words: RwSignal<HashSet<String>>,
    pub is_loading_preview: RwSignal<bool>,
    pub is_importing: RwSignal<bool>,
    pub error_message: RwSignal<Option<String>>,
    pub repository: HybridUserRepository,
    pub well_known_loader: WellKnownSetLoaderImpl,
    pub set_titles: RwSignal<HashMap<String, String>>,
    pub set_ids: RwSignal<Vec<String>>,
}

impl ImportPreviewModalState {
    pub fn new() -> Self {
        let repository =
            use_context::<HybridUserRepository>().expect("repository context not provided");

        let well_known_loader = WellKnownSetLoaderImpl::new();

        Self {
            preview_words: RwSignal::new(Vec::new()),
            selected_words: RwSignal::new(HashSet::new()),
            is_loading_preview: RwSignal::new(false),
            is_importing: RwSignal::new(false),
            error_message: RwSignal::new(None),
            repository,
            well_known_loader,
            set_titles: RwSignal::new(HashMap::new()),
            set_ids: RwSignal::new(Vec::new()),
        }
    }

    pub fn load_preview(&self, set_id: String) {
        let repository = self.repository.clone();
        let well_known_loader = self.well_known_loader.clone();
        let preview_words = self.preview_words;
        let selected_words = self.selected_words;
        let is_loading_preview = self.is_loading_preview;
        let error = self.error_message;
        let set_titles = self.set_titles;
        let set_ids = self.set_ids;

        preview_words.set(Vec::new());
        selected_words.set(HashSet::new());
        is_loading_preview.set(true);
        error.set(None);

        spawn_local(async move {
            let user = match repository.get_current_user().await {
                Ok(Some(u)) => u,
                Ok(None) => {
                    error.set(Some("Пользователь не найден".to_string()));
                    is_loading_preview.set(false);
                    return;
                },
                Err(e) => {
                    error.set(Some(e.to_string()));
                    is_loading_preview.set(false);
                    return;
                },
            };

            let set = match well_known_loader.load_set(set_id.clone()).await {
                Ok(s) => s,
                Err(e) => {
                    error.set(Some(e.to_string()));
                    is_loading_preview.set(false);
                    return;
                },
            };

            let set_title = set_ids
                .get()
                .first()
                .map(|id| {
                    set_titles
                        .get()
                        .get(id)
                        .cloned()
                        .unwrap_or_else(|| id.clone())
                })
                .unwrap_or_else(|| set_id.clone());

            let words = set.words();
            let mut words_preview = Vec::new();
            for word in words {
                let knowledge = user.is_word_known(word);
                words_preview.push(PreviewWord {
                    word: word.clone(),
                    meaning: knowledge.meaning,
                    is_known: knowledge.is_known,
                    set_id: set_id.clone(),
                    set_title: set_title.clone(),
                });
            }

            let words_to_select: HashSet<String> =
                words_preview.iter().map(|w| w.word.clone()).collect();
            preview_words.set(words_preview);
            selected_words.set(words_to_select);
            is_loading_preview.set(false);
        });
    }

    pub fn load_preview_multiple(
        &self,
        set_ids: Vec<String>,
        set_titles_input: HashMap<String, String>,
    ) {
        let repository = self.repository.clone();
        let well_known_loader = self.well_known_loader.clone();
        let preview_words = self.preview_words;
        let selected_words = self.selected_words;
        let is_loading_preview = self.is_loading_preview;
        let error = self.error_message;
        let set_titles = self.set_titles;
        let state_set_ids = self.set_ids;

        preview_words.set(Vec::new());
        selected_words.set(HashSet::new());
        is_loading_preview.set(true);
        error.set(None);
        set_titles.set(set_titles_input);
        state_set_ids.set(set_ids.clone());

        spawn_local(async move {
            let user = match repository.get_current_user().await {
                Ok(Some(u)) => u,
                Ok(None) => {
                    error.set(Some("Пользователь не найден".to_string()));
                    is_loading_preview.set(false);
                    return;
                },
                Err(e) => {
                    error.set(Some(e.to_string()));
                    is_loading_preview.set(false);
                    return;
                },
            };

            let sets_result = well_known_loader.load_sets(set_ids.clone()).await;
            let loaded_sets = match sets_result {
                Ok(sets) => sets,
                Err(e) => {
                    error.set(Some(e.to_string()));
                    is_loading_preview.set(false);
                    return;
                },
            };

            let mut words_preview = Vec::new();
            let titles = set_titles.get();

            for (set_id, set) in loaded_sets {
                let set_title = titles
                    .get(&set_id)
                    .cloned()
                    .unwrap_or_else(|| set_id.clone());

                for word in set.words() {
                    let knowledge = user.is_word_known(word);
                    words_preview.push(PreviewWord {
                        word: word.clone(),
                        meaning: knowledge.meaning,
                        is_known: knowledge.is_known,
                        set_id: set_id.clone(),
                        set_title: set_title.clone(),
                    });
                }
            }

            let words_to_select: HashSet<String> =
                words_preview.iter().map(|w| w.word.clone()).collect();
            preview_words.set(words_preview);
            selected_words.set(words_to_select);
            is_loading_preview.set(false);
        });
    }

    pub fn reset(&self) {
        self.preview_words.set(Vec::new());
        self.selected_words.set(HashSet::new());
        self.error_message.set(None);
        self.set_titles.set(HashMap::new());
        self.set_ids.set(Vec::new());
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
        let selected_words = self.selected_words.get();
        let preview_words = self.preview_words.get();

        let set_ids: Vec<String> = preview_words
            .iter()
            .filter(|pw| selected_words.contains(&pw.word))
            .map(|pw| pw.set_id.clone())
            .collect();

        let unique_set_ids: Vec<String> = {
            let mut unique = Vec::new();
            let mut seen = HashSet::new();
            for id in set_ids {
                if seen.insert(id.clone()) {
                    unique.push(id);
                }
            }
            unique
        };

        let words_to_create: Vec<WordToCreate> = selected_words
            .into_iter()
            .map(|base_form| WordToCreate { base_form })
            .collect();

        let repository = self.repository.clone();
        let is_importing = self.is_importing;
        let error = self.error_message;

        let set_ids_opt = if unique_set_ids.is_empty() {
            None
        } else {
            Some(unique_set_ids)
        };

        async move {
            is_importing.set(true);
            error.set(None);

            let use_case = CreateCardsFromAnalysisUseCase::new(&repository);
            match use_case.execute(words_to_create, set_ids_opt).await {
                Ok(result) => {
                    is_importing.set(false);
                    Ok(result)
                },
                Err(e) => {
                    error.set(Some(e.to_string()));
                    is_importing.set(false);
                    Err(e.to_string())
                },
            }
        }
    }
}
