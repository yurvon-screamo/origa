use crate::pages::sets::set_word_item::SetWordItem;
use crate::repository::HybridUserRepository;
use crate::ui_components::{
    Alert, AlertType, Button, ButtonVariant, Drawer, Spinner, Text, TextSize, ToastContainer,
    ToastData, TypographyVariant,
};
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::domain::User;
use origa::traits::UserRepository;

use super::import_set_preview_modal_handlers::create_import_preview_handlers;
use super::import_set_preview_modal_state::ImportPreviewModalState;

#[component]
pub fn ImportSetPreviewModal(
    is_open: RwSignal<bool>,
    set_id: Signal<String>,
    set_title: Signal<String>,
    on_import_result: Callback<()>,
) -> impl IntoView {
    let repository =
        use_context::<HybridUserRepository>().expect("repository context not provided");

    let current_user: RwSignal<Option<User>> = RwSignal::new(None);
    let repo_for_init = repository.clone();

    Effect::new(move |_| {
        let repo = repo_for_init.clone();
        spawn_local(async move {
            if let Ok(Some(user)) = repo.get_current_user().await {
                current_user.set(Some(user));
            }
        });
    });

    let known_kanji = Memo::new(move |_| {
        current_user
            .get()
            .map(|u| u.knowledge_set().get_known_kanji())
            .unwrap_or_default()
    });

    let state = ImportPreviewModalState::new();
    let toasts: RwSignal<Vec<ToastData>> = RwSignal::new(Vec::new());
    let handlers = create_import_preview_handlers(
        state.clone(),
        is_open,
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
        <Drawer
            is_open=is_open
            title=Signal::derive(move || set_title.get())
        >
            <div class="space-y-4">
                {move || {
                    let words = set_words.get();
                    let is_loading = is_loading_preview.get();

                    if let Some(error) = error_message.get() {
                        view! {
                            <Alert
                                alert_type=Signal::derive(|| AlertType::Error)
                                title=Signal::derive(|| "Ошибка".to_string())
                                message=Signal::derive(move || error.clone())
                            />
                        }.into_any()
                    } else if is_loading || words.is_empty() {
                        view! {
                            <div class="flex flex-col items-center py-4 gap-3">
                                <Spinner />
                                <Text size=TextSize::Default variant=TypographyVariant::Muted>
                                    "Загрузка списка слов..."
                                </Text>
                            </div>
                        }.into_any()
                    } else {
                        let known_count = words.iter().filter(|(_, _, known)| *known).count();

                        view! {
                            <Text size=TextSize::Small variant=TypographyVariant::Muted>
                                {format!("Найдено {} слов ({} известных)", words.len(), known_count)}
                            </Text>
                            <div class="space-y-2 overflow-y-auto">
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
                                                known_kanji=known_kanji.get()
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
        </Drawer>
    }
}
