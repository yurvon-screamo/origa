use crate::repository::HybridUserRepository;
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::dictionary::{RadicalInfo, get_radical_list};
use origa::traits::UserRepository;
use std::collections::HashSet;

#[derive(Clone)]
pub struct ModalState {
    #[allow(dead_code)]
    pub selected_level: RwSignal<String>,
    pub available_radicals: RwSignal<Vec<RadicalInfo>>,
    pub selected_radicals: RwSignal<HashSet<char>>,
    pub is_loading_radicals: RwSignal<bool>,
    pub is_creating: RwSignal<bool>,
    pub error_message: RwSignal<Option<String>>,
    pub repository: HybridUserRepository,
    pub refresh_trigger: RwSignal<u32>,
}

impl ModalState {
    pub fn new(is_open: RwSignal<bool>, refresh_trigger: RwSignal<u32>) -> Self {
        let repository =
            use_context::<HybridUserRepository>().expect("repository context not provided");

        let selected_radicals = RwSignal::new(HashSet::new());

        Effect::new({
            let selected_radicals_clone = selected_radicals;
            move |_| {
                if is_open.get() {
                    selected_radicals_clone.set(HashSet::new());
                }
            }
        });

        Self {
            selected_level: RwSignal::new("all".to_string()),
            available_radicals: RwSignal::new(Vec::new()),
            selected_radicals,
            is_loading_radicals: RwSignal::new(false),
            is_creating: RwSignal::new(false),
            error_message: RwSignal::new(None),
            repository,
            refresh_trigger,
        }
    }

    pub fn load_radicals(&self) {
        let repository = self.repository.clone();
        let available_radicals = self.available_radicals;
        let is_loading = self.is_loading_radicals;
        let error = self.error_message;

        is_loading.set(true);
        error.set(None);

        spawn_local(async move {
            match repository.get_current_user().await {
                Ok(Some(user)) => {
                    let learned_radicals: HashSet<char> = user
                        .knowledge_set()
                        .study_cards()
                        .iter()
                        .filter_map(|(_, card)| {
                            if let origa::domain::Card::Radical(radical_card) = card.card() {
                                Some(radical_card.radical_char())
                            } else {
                                None
                            }
                        })
                        .collect();

                    let all_radicals = get_radical_list();
                    let radical_list: Vec<RadicalInfo> = all_radicals
                        .into_iter()
                        .filter(|radical_info| !learned_radicals.contains(&radical_info.radical()))
                        .collect();

                    available_radicals.set(radical_list);
                    is_loading.set(false);
                }
                Ok(None) => {
                    error.set(Some("Пользователь не найден".to_string()));
                    is_loading.set(false);
                }
                Err(e) => {
                    error.set(Some(e.to_string()));
                    is_loading.set(false);
                }
            }
        });
    }

    #[allow(dead_code)]
    pub fn select_level(&self, level: String) {
        self.selected_level.set(level);
        self.selected_radicals.set(HashSet::new());
        self.load_radicals();
    }

    pub fn reset(&self) {
        self.error_message.set(None);
        self.selected_radicals.set(HashSet::new());
    }

    pub fn select_all(&self) {
        let all_radicals: HashSet<char> = self
            .available_radicals
            .get()
            .iter()
            .map(|r| r.radical())
            .collect();
        self.selected_radicals.set(all_radicals);
    }
}
