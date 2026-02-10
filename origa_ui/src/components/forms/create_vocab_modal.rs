use crate::components::cards::base_card::CardActions;
use crate::components::forms::bottom_sheet::BottomSheet;
use crate::components::forms::input::Input;
use leptos::prelude::*;

#[component]
pub fn CreateVocabularyModal(
    show: Signal<bool>,
    #[prop(into, optional)] on_close: Option<Callback<()>>,
    #[prop(into, optional)] on_create: Option<Callback<CreateVocabularyData>>,
) -> impl IntoView {
    let (japanese_text, set_japanese_text) = signal("".to_string());

    let (is_submitting, set_is_submitting) = signal(false);
    let (error, set_error) = signal(None::<String>);

    // Callback for passing to BottomSheet
    let on_close_callback = Callback::new(move |_| {
        if let Some(handler) = on_close {
            handler.run(());
        }
        // Reset form
        set_japanese_text.set("".to_string());
        set_error.set(None);
    });

    // Closure for button clicks
    let handle_close = move |_| {
        if let Some(handler) = on_close {
            handler.run(());
        }
        set_japanese_text.set("".to_string());
        set_error.set(None);
    };

    let handle_create = move |_| {
        let japanese = japanese_text.get();

        // Validation
        if japanese.trim().is_empty() {
            set_error.set(Some("Японский текст обязателен".to_string()));
            return;
        }

        // Create data
        let data = CreateVocabularyData {
            japanese: japanese.trim().to_string(),
        };

        set_is_submitting.set(true);
        set_error.set(None);

        set_is_submitting.set(false);

        if let Some(handler) = on_create {
            handler.run(data.clone());
        }

        // Close modal
        if let Some(handler) = on_close {
            handler.run(());
        }
        // Reset form
        set_japanese_text.set("".to_string());
        set_error.set(None);
    };

    let is_form_valid =
        Signal::derive(move || !japanese_text.get().trim().is_empty() && !is_submitting.get());

    view! {
        <BottomSheet
            show=show
            title="Добавить слово"
            subtitle="Введите японское слово и его перевод"
            on_close=on_close_callback
        >
            <div class="create-vocab-form">
                <Input
                    label="Японский текст"
                    placeholder="例: 本"
                    value=japanese_text
                    on_change=Callback::new(move |val| set_japanese_text.set(val))
                    required=true
                    maxlength=50u32
                />

                // Error display
                <Show when=move || error.get().is_some()>
                    <div class="form-error">{move || error.get().unwrap_or_default()}</div>
                </Show>

                // Action buttons
                <CardActions>
                    <button
                        class="button button-subtle"
                        on:click=handle_close
                        disabled=move || is_submitting.get()
                    >
                        "Отмена"
                    </button>
                    <button
                        class="button button-primary"
                        on:click=handle_create
                        disabled=move || !is_form_valid.get()
                    >
                        {move || {
                            if is_submitting.get() {
                                view! {
                                    <span class="loading-spinner"></span>
                                    <span>"Создание..."</span>
                                }
                                    .into_any()
                            } else {
                                view! { <span>"Добавить слово"</span> }.into_any()
                            }
                        }}
                    </button>
                </CardActions>

            </div>
        </BottomSheet>
    }
}

#[derive(Clone)]
pub struct CreateVocabularyData {
    pub japanese: String,
}
