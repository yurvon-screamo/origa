use crate::ui_components::{ToastData, ToastType};
use leptos::ev::MouseEvent;
use leptos::prelude::*;
use leptos::task::spawn_local;

use super::import_set_preview_modal_state::ImportPreviewModalState;

#[derive(Clone)]
pub struct ImportPreviewHandlers {
    pub on_word_toggle: Callback<String>,
    pub on_import: Callback<()>,
    pub on_cancel: Callback<MouseEvent>,
}

pub fn create_import_preview_handlers(
    state: ImportPreviewModalState,
    is_open: RwSignal<bool>,
    toasts: RwSignal<Vec<ToastData>>,
    on_import_result: Callback<Vec<String>>,
) -> ImportPreviewHandlers {
    let state_clone = state.clone();
    let on_word_toggle = Callback::new(move |word: String| {
        state_clone.toggle_word(word);
    });

    let state_clone = state.clone();
    let is_open_clone = is_open;
    let toasts_clone = toasts;
    let on_import_result_clone = on_import_result;
    let disposed = state.disposed;
    let on_import = Callback::new(move |_: ()| {
        let selected = state_clone.selected_words.get();
        if selected.is_empty() {
            return;
        }

        let state = state_clone.clone();
        let is_open = is_open_clone;
        let toasts = toasts_clone;
        let on_import_result = on_import_result_clone;

        state.is_importing.set(true);

        let imported_set_ids = state
            .preview_words
            .get()
            .iter()
            .map(|w| w.set_id.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();

        spawn_local(async move {
            match state.import_selected().await {
                Ok(result) => {
                    if disposed.is_disposed() {
                        return;
                    }
                    state.is_importing.set(false);
                    state.reset();
                    is_open.set(false);
                    on_import_result.run(imported_set_ids);

                    let toast_id = toasts.get().len();
                    let message = if result.failed_words.is_empty() {
                        if result.skipped_words.is_empty() {
                            format!("Успешно создано {} карточек", result.created_cards.len())
                        } else {
                            format!(
                                "Создано {} карточек, пропущено {} (уже существуют)",
                                result.created_cards.len(),
                                result.skipped_words.len()
                            )
                        }
                    } else {
                        format!(
                            "Создано {} карточек, пропущено {}, ошибок: {}",
                            result.created_cards.len(),
                            result.skipped_words.len(),
                            result.failed_words.len()
                        )
                    };

                    toasts.update(|t| {
                        t.push(ToastData {
                            id: toast_id,
                            title: "Импорт завершён".to_string(),
                            message,
                            toast_type: ToastType::Success,
                            duration_ms: None,
                            closable: true,
                        });
                    });
                },
                Err(e) => {
                    if disposed.is_disposed() {
                        return;
                    }
                    state.is_importing.set(false);
                    state.error_message.set(Some(e.clone()));
                    on_import_result.run(Vec::new());
                    let toast_id = toasts.get().len();
                    toasts.update(|t| {
                        t.push(ToastData {
                            id: toast_id,
                            title: "Ошибка".to_string(),
                            message: e,
                            toast_type: ToastType::Error,
                            duration_ms: None,
                            closable: true,
                        });
                    });
                },
            }
        });
    });

    let state_clone = state.clone();
    let is_open_clone = is_open;
    let on_cancel = Callback::new(move |_: MouseEvent| {
        state_clone.reset();
        is_open_clone.set(false);
    });

    ImportPreviewHandlers {
        on_word_toggle,
        on_import,
        on_cancel,
    }
}
