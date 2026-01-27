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
    let (translation, set_translation) = signal("".to_string());
    let (reading, set_reading) = signal("".to_string());
    let (notes, set_notes) = signal("".to_string());

    let (is_submitting, set_is_submitting) = signal(false);
    let (error, set_error) = signal(None::<String>);

    // Callback for passing to BottomSheet
    let on_close_callback = Callback::new(move |_| {
        if let Some(handler) = on_close {
            handler.run(());
        }
        // Reset form
        set_japanese_text.set("".to_string());
        set_translation.set("".to_string());
        set_reading.set("".to_string());
        set_notes.set("".to_string());
        set_error.set(None);
    });

    // Closure for button clicks
    let handle_close = move |_| {
        if let Some(handler) = on_close {
            handler.run(());
        }
        set_japanese_text.set("".to_string());
        set_translation.set("".to_string());
        set_reading.set("".to_string());
        set_notes.set("".to_string());
        set_error.set(None);
    };

    let handle_create = move |_| {
        let japanese = japanese_text.get();
        let trans = translation.get();

        // Validation
        if japanese.trim().is_empty() {
            set_error.set(Some("Японский текст обязателен".to_string()));
            return;
        }

        if trans.trim().is_empty() {
            set_error.set(Some("Перевод обязателен".to_string()));
            return;
        }

        // Create data
        let data = CreateVocabularyData {
            japanese: japanese.trim().to_string(),
            translation: trans.trim().to_string(),
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
        set_translation.set("".to_string());
        set_reading.set("".to_string());
        set_notes.set("".to_string());
        set_error.set(None);
    };

    let is_form_valid = Signal::derive(move || {
        !japanese_text.get().trim().is_empty()
            && !translation.get().trim().is_empty()
            && !is_submitting.get()
    });

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

                <Input
                    label="Чтение (фуригана)"
                    placeholder="例: ほん"
                    value=reading
                    on_change=Callback::new(move |val| set_reading.set(val))
                    maxlength=50u32
                />

                <Input
                    label="Перевод"
                    placeholder="例: книга"
                    value=translation
                    on_change=Callback::new(move |val| set_translation.set(val))
                    required=true
                    maxlength=100u32
                />

                <Input
                    label="Примечания (необязательно)"
                    placeholder="Дополнительная информация о слове"
                    value=notes
                    on_change=Callback::new(move |val| set_notes.set(val))
                    multiline=true
                    rows=3u32
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

                // Help text
                <div class="form-help">
                    <p class="help-text">
                        "💡 Совет: Если вы не знаете чтение, оставьте поле пустым. Система автоматически сгенерирует фуригану."
                    </p>
                </div>
            </div>
        </BottomSheet>
    }
}

#[derive(Clone)]
pub struct CreateVocabularyData {
    pub japanese: String,
    pub translation: String,
}

#[component]
pub fn VocabularyCreationTips() -> impl IntoView {
    view! {
        <div class="vocab-tips">
            <h3 class="tips-title">Советы по добавлению слов</h3>

            <div class="tip-item">
                <span class="tip-icon">{"📝"}</span>
                <div class="tip-content">
                    <h4 class="tip-heading">Используйте канжи</h4>
                    <p class="tip-text">
                        Добавляйте слова в канзи, а не в хирагане. Это поможет лучше запомнить написание.
                    </p>
                </div>
            </div>

            <div class="tip-item">
                <span class="tip-icon">{"🔊"}</span>
                <div class="tip-content">
                    <h4 class="tip-heading">Правильное чтение</h4>
                    <p class="tip-text">
                        Указывайте точное чтение (онъоми/кунъоми) для лучшего запоминания произношения.
                    </p>
                </div>
            </div>

            <div class="tip-item">
                <span class="tip-icon">{"📚"}</span>
                <div class="tip-content">
                    <h4 class="tip-heading">Контекст важен</h4>
                    <p class="tip-text">
                        Добавляйте примеры использования в примечаниях для лучшего понимания контекста.
                    </p>
                </div>
            </div>

            <div class="tip-item">
                <span class="tip-icon">{"🎯"}</span>
                <div class="tip-content">
                    <h4 class="tip-heading">Маленькими порциями</h4>
                    <p class="tip-text">
                        Добавляйте 5-10 слов за раз для лучшего запоминания и регулярного повторения.
                    </p>
                </div>
            </div>
        </div>
    }
}
