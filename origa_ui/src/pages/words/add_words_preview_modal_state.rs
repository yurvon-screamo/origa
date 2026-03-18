use crate::repository::HybridUserRepository;
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::use_cases::{
    AnalyzeTextForCardsUseCase, AnalyzedWord, CreateCardsFromAnalysisResult,
    CreateCardsFromAnalysisUseCase, WordToCreate,
};
use std::collections::HashSet;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum InputMode {
    #[default]
    Text,
    Image,
}

#[derive(Clone)]
pub struct PreviewModalState {
    pub input_mode: RwSignal<InputMode>,
    pub active_tab: RwSignal<String>,
    pub input_text: RwSignal<String>,
    pub analyzed_words: RwSignal<Vec<AnalyzedWord>>,
    pub selected_words: RwSignal<HashSet<String>>,
    pub is_analyzing: RwSignal<bool>,
    pub is_creating: RwSignal<bool>,
    pub error_message: RwSignal<Option<String>>,
    pub repository: HybridUserRepository,
    pub refresh_trigger: RwSignal<u32>,
}

impl PreviewModalState {
    pub fn new(is_open: RwSignal<bool>, refresh_trigger: RwSignal<u32>) -> Self {
        let repository =
            use_context::<HybridUserRepository>().expect("repository context not provided");

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
            input_mode: RwSignal::new(InputMode::Text),
            active_tab: RwSignal::new("text".to_string()),
            input_text: RwSignal::new(String::new()),
            analyzed_words: RwSignal::new(Vec::new()),
            selected_words,
            is_analyzing: RwSignal::new(false),
            is_creating: RwSignal::new(false),
            error_message: RwSignal::new(None),
            repository,
            refresh_trigger,
        }
    }

    pub fn analyze_text(&self) {
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
            match use_case.execute(text).await {
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

    pub fn set_extracted_text(&self, text: String) {
        self.input_text.set(text);
        self.analyze_text();
    }

    pub fn reset(&self) {
        self.input_mode.set(InputMode::Text);
        self.active_tab.set("text".to_string());
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
        let selected_words = self.selected_words.get();
        let words_to_create: Vec<WordToCreate> = selected_words
            .into_iter()
            .map(|base_form| WordToCreate { base_form })
            .collect();
        let repository = self.repository.clone();
        let is_creating = self.is_creating;
        let error = self.error_message;

        async move {
            is_creating.set(true);
            error.set(None);

            let use_case = CreateCardsFromAnalysisUseCase::new(&repository);
            match use_case.execute(words_to_create, None).await {
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
