use super::add_kanji_modal_state::ModalState;
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::use_cases::CreateKanjiCardUseCase;

pub struct ModalHandlers {
    pub on_add: Callback<leptos::ev::MouseEvent>,
}

impl ModalHandlers {
    pub fn new(state: &ModalState, is_open: RwSignal<bool>) -> Self {
        let on_add = {
            let state = state.clone();
            Callback::new(move |_| {
                let kanji_list: Vec<String> = state.selected_kanji.get().into_iter().collect();
                if kanji_list.is_empty() {
                    return;
                }

                let repository = state.repository.clone();
                let is_creating = state.is_creating;
                let error = state.error_message;
                let state_for_async = state.clone();
                let is_open_for_async = is_open;

                is_creating.set(true);
                error.set(None);
                let disposed = StoredValue::new(());

                spawn_local(async move {
                    let use_case = CreateKanjiCardUseCase::new(&repository);
                    match use_case.execute(kanji_list).await {
                        Ok(_) => {
                            if disposed.is_disposed() {
                                return;
                            }
                            is_creating.set(false);
                            state_for_async.reset();
                            is_open_for_async.set(false);
                            state_for_async.refresh_trigger.update(|v| *v += 1);
                        },
                        Err(e) => {
                            if disposed.is_disposed() {
                                return;
                            }
                            is_creating.set(false);
                            error.set(Some(e.to_string()));
                        },
                    }
                });
            })
        };

        Self { on_add }
    }
}
