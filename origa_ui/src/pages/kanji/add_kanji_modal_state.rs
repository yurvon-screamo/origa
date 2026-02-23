use crate::repository::SupabaseUserRepository;
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::application::{KanjiInfoListUseCase, KanjiItemInfo};
use origa::domain::{JapaneseLevel, User};
use std::collections::HashSet;

#[derive(Clone)]
pub struct ModalState {
    pub selected_level: RwSignal<JapaneseLevel>,
    pub available_kanji: RwSignal<Vec<KanjiItemInfo>>,
    pub selected_kanji: RwSignal<HashSet<String>>,
    pub is_loading_kanji: RwSignal<bool>,
    pub is_creating: RwSignal<bool>,
    pub error_message: RwSignal<Option<String>>,
    pub current_user: RwSignal<Option<User>>,
    pub repository: SupabaseUserRepository,
}

impl ModalState {
    pub fn new(is_open: RwSignal<bool>) -> Self {
        let current_user =
            use_context::<RwSignal<Option<User>>>().expect("current_user context not provided");
        let repository =
            use_context::<SupabaseUserRepository>().expect("repository context not provided");

        let selected_kanji = RwSignal::new(HashSet::new());

        Effect::new({
            let selected_kanji_clone = selected_kanji;
            move |_| {
                if is_open.get() {
                    selected_kanji_clone.set(HashSet::new());
                }
            }
        });

        Self {
            selected_level: RwSignal::new(JapaneseLevel::N5),
            available_kanji: RwSignal::new(Vec::new()),
            selected_kanji,
            is_loading_kanji: RwSignal::new(false),
            is_creating: RwSignal::new(false),
            error_message: RwSignal::new(None),
            current_user,
            repository,
        }
    }

    pub fn load_kanji(&self) {
        let user_id = self
            .current_user
            .with(|u| u.as_ref().map(|u| u.id()))
            .unwrap();
        let level = self.selected_level.get();
        let repository = self.repository.clone();
        let available_kanji = self.available_kanji;
        let is_loading = self.is_loading_kanji;
        let error = self.error_message;

        is_loading.set(true);
        error.set(None);

        spawn_local(async move {
            let use_case = KanjiInfoListUseCase::new(&repository);
            match use_case.execute(user_id, &level).await {
                Ok(kanji_list) => {
                    available_kanji.set(kanji_list);
                    is_loading.set(false);
                }
                Err(e) => {
                    error.set(Some(e.to_string()));
                    is_loading.set(false);
                }
            }
        });
    }

    pub fn select_level(&self, level: JapaneseLevel) {
        self.selected_level.set(level);
        self.selected_kanji.set(HashSet::new());
        self.load_kanji();
    }

    pub fn reset(&self) {
        self.error_message.set(None);
        self.selected_kanji.set(HashSet::new());
    }
}
