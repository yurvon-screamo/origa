use crate::repository::HybridUserRepository;
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::dictionary::counters::{CounterEntry, counters_up_to_level};
use origa::domain::{Card, JapaneseLevel};
use origa::traits::UserRepository;
use std::collections::HashSet;
use tracing::error;

#[derive(Clone)]
pub struct DrawerState {
    pub selected_level: RwSignal<JapaneseLevel>,
    pub available_counters: RwSignal<Vec<&'static CounterEntry>>,
    pub selected_counters: RwSignal<HashSet<String>>,
    pub is_loading_counters: RwSignal<bool>,
    pub is_creating: RwSignal<bool>,
    pub error_message: RwSignal<Option<String>>,
    pub repository: HybridUserRepository,
    pub refresh_trigger: RwSignal<u32>,
    pub search_query: RwSignal<String>,
}

impl DrawerState {
    pub fn new(is_open: RwSignal<bool>, refresh_trigger: RwSignal<u32>) -> Self {
        let repository =
            use_context::<HybridUserRepository>().expect("repository context not provided");

        let selected_counters = RwSignal::new(HashSet::new());

        Effect::new(move |_| {
            if is_open.get() {
                selected_counters.set(HashSet::new());
            }
        });

        Self {
            selected_level: RwSignal::new(JapaneseLevel::N5),
            available_counters: RwSignal::new(Vec::new()),
            selected_counters,
            is_loading_counters: RwSignal::new(false),
            is_creating: RwSignal::new(false),
            error_message: RwSignal::new(None),
            repository,
            refresh_trigger,
            search_query: RwSignal::new(String::new()),
        }
    }

    /// Реестр уровня минус суффиксы, уже заведённые в колоду (зеркало
    /// `ModalState::load_kanji` кандзи-дровера).
    pub fn load_counters(&self) {
        let level = self.selected_level.get();
        let repository = self.repository.clone();
        let available_counters = self.available_counters;
        let is_loading = self.is_loading_counters;
        let error = self.error_message;
        let disposed = StoredValue::new(());

        is_loading.set(true);
        error.set(None);

        spawn_local(async move {
            match repository.get_current_user().await {
                Ok(Some(user)) => {
                    if disposed.is_disposed() {
                        return;
                    }
                    let in_deck: HashSet<String> = user
                        .knowledge_set()
                        .study_cards()
                        .values()
                        .filter_map(|card| match card.card() {
                            Card::Counter(counter) => Some(counter.suffix().to_string()),
                            _ => None,
                        })
                        .collect();
                    let list: Vec<&'static CounterEntry> = counters_up_to_level(level)
                        .into_iter()
                        .filter(|entry| !in_deck.contains(entry.suffix()))
                        .collect();
                    available_counters.set(list);
                    is_loading.set(false);
                },
                Ok(None) => {
                    if disposed.is_disposed() {
                        return;
                    }
                    error.set(Some(
                        crate::i18n::use_i18n()
                            .get_keys_untracked()
                            .shared()
                            .user_not_found()
                            .inner()
                            .to_string(),
                    ));
                    is_loading.set(false);
                },
                Err(e) => {
                    error!(error = %e, "Counters drawer user data load failed");
                    if disposed.is_disposed() {
                        return;
                    }
                    error.set(Some(e.to_string()));
                    is_loading.set(false);
                },
            }
        });
    }

    pub fn select_level(&self, level: JapaneseLevel) {
        self.selected_level.set(level);
        self.selected_counters.set(HashSet::new());
        self.search_query.set(String::new());
        self.load_counters();
    }

    pub fn reset(&self) {
        self.error_message.set(None);
        self.selected_counters.set(HashSet::new());
    }

    pub fn select_all(&self) {
        let all: HashSet<String> = self
            .available_counters
            .get()
            .iter()
            .map(|entry| entry.suffix().to_string())
            .collect();
        self.selected_counters.set(all);
    }
}
