use super::add_words_preview_modal_state::PreviewModalState;
use leptos::ev::MouseEvent;
use leptos::prelude::*;
use leptos::task::spawn_local;

pub struct PreviewModalHandlers {
    pub on_analyze: Callback<()>,
    pub on_word_toggle: Callback<String>,
    pub on_create: Callback<()>,
    pub on_cancel: Callback<MouseEvent>,
}

pub fn create_preview_modal_handlers(
    state: PreviewModalState,
    is_open: RwSignal<bool>,
) -> PreviewModalHandlers {
    let on_analyze = {
        let state = state.clone();
        Callback::new(move |_| {
            state.analyze_text();
        })
    };

    let on_word_toggle = {
        let state = state.clone();
        Callback::new(move |word| {
            state.toggle_word(word);
        })
    };

    let on_create = {
        let state = state.clone();
        let disposed = state.disposed;
        Callback::new(move |_| {
            let selected_words_count = state.selected_words.get_untracked().len();
            if selected_words_count == 0 {
                return;
            }

            let is_creating = state.is_creating;
            let error = state.error_message;
            let state_for_async = state.clone();
            let is_open_for_async = is_open;

            is_creating.set(true);
            error.set(None);

            spawn_local(async move {
                match state_for_async.create_cards().await {
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

    let on_cancel = {
        Callback::new(move |_| {
            state.reset();
            is_open.set(false);
        })
    };

    PreviewModalHandlers {
        on_analyze,
        on_word_toggle,
        on_create,
        on_cancel,
    }
}
