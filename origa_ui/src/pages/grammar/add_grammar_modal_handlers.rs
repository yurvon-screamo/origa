use super::add_grammar_modal_state::ModalState;
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::use_cases::CreateGrammarCardUseCase;
use ulid::Ulid;

pub struct ModalHandlers {
    pub on_cancel: Callback<leptos::ev::MouseEvent>,
    pub on_add: Callback<leptos::ev::MouseEvent>,
}

impl ModalHandlers {
    pub fn new(
        state: &ModalState,
        is_open: RwSignal<bool>,
        refresh_trigger: RwSignal<u32>,
    ) -> Self {
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
                let rule_ids: Vec<Ulid> = state.selected_rule_ids.get().into_iter().collect();
                if rule_ids.is_empty() {
                    return;
                }

                let repository = state.repository.clone();
                let is_creating = state.is_creating;
                let error = state.error_message;
                let state_for_async = state.clone();
                let is_open_for_async = is_open;
                let refresh_for_async = refresh_trigger;

                is_creating.set(true);
                error.set(None);

                spawn_local(async move {
                    let use_case = CreateGrammarCardUseCase::new(&repository);
                    match use_case.execute(rule_ids).await {
                        Ok(_) => {
                            is_creating.set(false);
                            state_for_async.reset();
                            is_open_for_async.set(false);
                            refresh_for_async.update(|v| *v += 1);
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
