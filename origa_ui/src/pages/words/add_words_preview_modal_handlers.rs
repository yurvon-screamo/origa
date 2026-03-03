use super::add_words_preview_modal_state::PreviewModalState;
use crate::app::update_current_user;
use crate::repository::HybridUserRepository;
use leptos::ev::MouseEvent;
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::domain::User;

pub struct PreviewModalHandlers {
    pub on_analyze: Callback<()>,
    pub on_word_toggle: Callback<String>,
    pub on_create: Callback<()>,
    pub on_cancel: Callback<MouseEvent>,
}

pub fn create_preview_modal_handlers(
    state: PreviewModalState,
    is_open: RwSignal<bool>,
    current_user: RwSignal<Option<User>>,
    repository: HybridUserRepository,
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
        Callback::new(move |_| {
            let selected_words_count = state.selected_words.get().len();
            if selected_words_count == 0 {
                return;
            }

            let is_creating = state.is_creating;
            let error = state.error_message;
            let state_for_async = state.clone();
            let is_open_for_async = is_open;
            let repository_for_async = repository.clone();
            let current_user_for_async = current_user;

            is_creating.set(true);
            error.set(None);

            spawn_local(async move {
                match state_for_async.create_cards().await {
                    Ok(_) => {
                        is_creating.set(false);
                        update_current_user(repository_for_async, current_user_for_async);
                        state_for_async.reset();
                        is_open_for_async.set(false);
                    }
                    Err(e) => {
                        is_creating.set(false);
                        error.set(Some(e));
                    }
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
