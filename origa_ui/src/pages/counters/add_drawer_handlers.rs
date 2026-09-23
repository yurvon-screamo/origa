use super::add_drawer_state::DrawerState;
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::use_cases::AddCounterCardsUseCase;
use tracing::error;

pub struct DrawerHandlers {
    pub on_add: Callback<leptos::ev::MouseEvent>,
}

impl DrawerHandlers {
    pub fn new(state: &DrawerState, is_open: RwSignal<bool>) -> Self {
        let on_add = {
            let state = state.clone();
            Callback::new(move |_| {
                let suffixes: Vec<String> = state.selected_counters.get().into_iter().collect();
                if suffixes.is_empty() {
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
                    let use_case = AddCounterCardsUseCase::new(&repository);
                    match use_case.execute(suffixes).await {
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
                            error!(error = %e, "Counter card creation failed");
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
