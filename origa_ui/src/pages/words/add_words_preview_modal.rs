use crate::pages::words::add_words_preview_modal_handlers::create_preview_modal_handlers;
use crate::pages::words::add_words_preview_modal_state::PreviewModalState;
use crate::pages::words::analyzed_word_item::AnalyzedWordItem;
use crate::ui_components::{
    Alert, AlertType, Button, ButtonVariant, Input, Modal, Text, TextSize, TypographyVariant,
};
use leptos::ev::MouseEvent;
use leptos::prelude::*;
use origa::application::AnalyzedWord;

#[component]
pub fn AddWordsPreviewModal(is_open: RwSignal<bool>) -> impl IntoView {
    let state = PreviewModalState::new(is_open);
    let current_user = state.current_user;
    let repository = state.repository.clone();
    let analyzed_words = state.analyzed_words;
    let input_text = state.input_text;
    let is_analyzing = state.is_analyzing;
    let error_message = state.error_message;
    let selected_words = state.selected_words;
    let is_creating = state.is_creating;
    let handlers = create_preview_modal_handlers(state, is_open, current_user, repository);

    view! {
        <Modal
            is_open=is_open
            title=Signal::derive(|| "Добавить слова из текста".to_string())
        >
            <div class="space-y-4">
                {move || {
                    let words = analyzed_words.get();
                    if words.is_empty() {
                        view! {
                            <InputStage
                                input_text=input_text
                                is_analyzing=is_analyzing
                                error_message=error_message
                                on_analyze=handlers.on_analyze.clone()
                            />
                        }.into_any()
                    } else {
                        view! {
                            <PreviewStage
                                analyzed_words=words
                                selected_words=selected_words
                                is_creating=is_creating
                                on_word_toggle=handlers.on_word_toggle.clone()
                                on_cancel=handlers.on_cancel.clone()
                                on_create=handlers.on_create.clone()
                            />
                        }.into_any()
                    }
                }}
            </div>
        </Modal>
    }
}

#[component]
fn PreviewStage(
    analyzed_words: Vec<AnalyzedWord>,
    selected_words: RwSignal<std::collections::HashSet<String>>,
    is_creating: RwSignal<bool>,
    on_word_toggle: Callback<String>,
    on_cancel: Callback<MouseEvent>,
    on_create: Callback<()>,
) -> impl IntoView {
    let analyzed_words_count = analyzed_words.len();
    let new_words_count = analyzed_words.iter().filter(|w| !w.is_known).count();

    view! {
        <div>
            <Text size=TextSize::Small variant=TypographyVariant::Muted>
                {format!(
                    "Найдено {} слов ({} новых)",
                    analyzed_words_count,
                    new_words_count
                )}
            </Text>
        </div>
        <div class="space-y-2 max-h-64 overflow-y-auto">
            <For
                each=move || analyzed_words.clone()
                key=|word| word.base_form.clone()
                children=move |word| {
                    let base_form = word.base_form.clone();
                    view! {
                        <AnalyzedWordItem
                            analyzed_word=word
                            selected_words=selected_words
                            on_toggle=Callback::new(move |_| on_word_toggle.run(base_form.clone()))
                        />
                    }
                }
            />
        </div>
        <div class="flex gap-2 justify-between">
            <Button
                variant=ButtonVariant::Ghost
                on_click=on_cancel
            >
                "Отмена"
            </Button>
            <Button
                variant=ButtonVariant::Olive
                disabled=Signal::derive(move || {
                    selected_words.get().is_empty()
                        || is_creating.get()
                })
                on_click=Callback::new(move |_| on_create.run(()))
            >
                {move || {
                    if is_creating.get() {
                        "Создание..."
                    } else {
                        "Добавить выбранные"
                    }
                }}
            </Button>
        </div>
    }
}

#[component]
fn InputStage(
    input_text: RwSignal<String>,
    is_analyzing: RwSignal<bool>,
    error_message: RwSignal<Option<String>>,
    on_analyze: Callback<()>,
) -> impl IntoView {
    view! {
        <div>
            <Text size=TextSize::Small variant=TypographyVariant::Muted class=Signal::derive(|| "mb-2".to_string())>
                "Введите текст на японском языке"
            </Text>
            <Input
                value=input_text
                placeholder=Signal::derive(|| "例えば、本を読みます。".to_string())
                rows=Signal::derive(|| Some(6))
            />
        </div>
        {move || {
            error_message.get().map(|msg| view! {
                <Alert
                    alert_type=Signal::derive(|| AlertType::Error)
                    title=Signal::derive(|| "Ошибка".to_string())
                    message=Signal::derive(move || msg.clone())
                />
            })
        }}
        <Button
            variant=ButtonVariant::Olive
            disabled=Signal::derive(move || {
                input_text.get().trim().is_empty()
                    || is_analyzing.get()
            })
            on_click=Callback::new(move |_| on_analyze.run(()))
        >
            {move || {
                if is_analyzing.get() {
                    "Анализ..."
                } else {
                    "Анализировать"
                }
            }}
        </Button>
    }
}
