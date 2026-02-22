use super::add_kanji_modal_state::ModalState;
use crate::app::update_current_user;
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::application::CreateKanjiCardUseCase;

pub struct ModalHandlers {
    pub on_cancel: Callback<leptos::ev::MouseEvent>,
    pub on_add: Callback<leptos::ev::MouseEvent>,
}

impl ModalHandlers {
    pub fn new(state: &ModalState, is_open: RwSignal<bool>) -> Self {
        let on_cancel = {
            let state = state.clone();
            Callback::new(move |_| {
                state.reset();
                is_open.set(false);
            })
        };

        let on_add = {
            let state = state.clone();
            Callback::new(move |_| {
                let kanji_list: Vec<String> = state.selected_kanji.get().into_iter().collect();
                if kanji_list.is_empty() {
                    return;
                }

                let user_id = state
                    .current_user
                    .with(|u| u.as_ref().map(|u| u.id()))
                    .unwrap();
                let repository = state.repository.clone();
                let is_creating = state.is_creating;
                let error = state.error_message;
                let state_for_async = state.clone();
                let is_open_for_async = is_open;

                is_creating.set(true);
                error.set(None);

                spawn_local(async move {
                    let use_case = CreateKanjiCardUseCase::new(&repository);
                    match use_case.execute(user_id, kanji_list).await {
                        Ok(_) => {
                            is_creating.set(false);
                            update_current_user(repository, state_for_async.current_user);
                            state_for_async.reset();
                            is_open_for_async.set(false);
                        }
                        Err(e) => {
                            is_creating.set(false);
                            error.set(Some(e.to_string()));
                        }
                    }
                });
            })
        };

        Self { on_cancel, on_add }
    }
}
