use crate::pages::sets::set_word_item::SetWordItem;
use crate::ui_components::{
    Alert, AlertType, Button, ButtonVariant, Modal, Text, TextSize, ToastContainer, ToastData,
    TypographyVariant,
};
use leptos::prelude::*;

use super::import_set_preview_modal_handlers::create_import_preview_handlers;
use super::import_set_preview_modal_handlers::ImportResult;
use super::import_set_preview_modal_state::ImportPreviewModalState;

#[component]
pub fn ImportSetPreviewModal(
    is_open: RwSignal<bool>,
    set_id: Signal<String>,
    set_title: Signal<String>,
    on_import_result: Callback<ImportResult>,
) -> impl IntoView {
    let state = ImportPreviewModalState::new();
    let current_user = state.current_user;
    let repository = state.repository.clone();
    let toasts: RwSignal<Vec<ToastData>> = RwSignal::new(Vec::new());
    let handlers = create_import_preview_handlers(
        state.clone(),
        is_open,
        current_user,
        repository,
        toasts,
        on_import_result,
    );

    let set_words = state.set_words;
    let selected_words = state.selected_words;
    let is_loading_preview = state.is_loading_preview;
    let is_importing = state.is_importing;
    let error_message = state.error_message;

    Effect::new({
        let state = state.clone();
        move |_| {
            if is_open.get() {
                let id = set_id.get();
                state.set_set_id(id.clone());
                state.load_preview(id);
            }
        }
    });

    view! {
        <Modal
            is_open=is_open
            title=Signal::derive(move || format!("Импортировать: {}", set_title.get()))
        >
            <div class="space-y-4">
                {move || {
                    let words = set_words.get();
                    let is_loading = is_loading_preview.get();

                    if is_loading {
                        view! {
                            <Text size=TextSize::Default variant=TypographyVariant::Muted>
                                "Загрузка списка слов..."
                            </Text>
                        }.into_any()
                    } else if let Some(error) = error_message.get() {
                        view! {
                            <Alert
                                alert_type=Signal::derive(|| AlertType::Error)
                                title=Signal::derive(|| "Ошибка".to_string())
                                message=Signal::derive(move || error.clone())
                            />
                        }.into_any()
                    } else if words.is_empty() {
                        view! {
                            <Text size=TextSize::Default variant=TypographyVariant::Muted>
                                "Загрузка списка слов..."
                            </Text>
                        }.into_any()
                    } else {
                        let known_count = words.iter().filter(|(_, _, known)| *known).count();

                        view! {
                            <Text size=TextSize::Small variant=TypographyVariant::Muted>
                                {format!("Найдено {} слов ({} известных)", words.len(), known_count)}
                            </Text>
                            <div class="space-y-2 max-h-64 overflow-y-auto">
                                <For
                                    each=move || words.clone()
                                    key=|word| word.0.clone()
                                    children=move |word| {
                                        let word_text = word.0.clone();
                                        let known_meaning = word.1.clone();
                                        let is_known = word.2;

                                        view! {
                                            <SetWordItem
                                                word=word_text.clone()
                                                known_meaning=known_meaning
                                                is_known=is_known
                                                selected_words=selected_words
                                                on_toggle=Callback::new(move |_| handlers.on_word_toggle.run(word_text.clone()))
                                            />
                                        }
                                    }
                                />
                            </div>
                            <div class="flex gap-2 justify-between">
                                <Button
                                    variant=ButtonVariant::Ghost
                                    on_click=handlers.on_cancel
                                >
                                    "Отмена"
                                </Button>
                                <Button
                                    variant=ButtonVariant::Olive
                                    disabled=Signal::derive(move || {
                                        selected_words.get().is_empty()
                                            || is_importing.get()
                                    })
                                    on_click=Callback::new(move |_| handlers.on_import.run(()))
                                >
                                    {move || {
                                        if is_importing.get() {
                                            "Импортирование..."
                                        } else {
                                            "Импортировать"
                                        }
                                    }}
                                </Button>
                            </div>
                        }.into_any()
                    }
                }}
            </div>
            <ToastContainer toasts=toasts duration_ms=5000 />
        </Modal>
    }
}
