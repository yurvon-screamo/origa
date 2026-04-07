use crate::repository::HybridUserRepository;
use crate::ui_components::{
    Alert, AlertType, Button, ButtonVariant, Dropdown, DropdownItem, Spinner, Text, TextSize,
    TypographyVariant,
};
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::use_cases::{
    AnkiCard, AnkiFieldInfo, ImportAnkiPackUseCase, extract_cards, read_anki_database,
};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use web_sys::js_sys::Function;
use web_sys::{File, HtmlInputElement};

const MAX_APKG_SIZE_BYTES: u64 = 50 * 1024 * 1024;

#[derive(Clone, Copy, PartialEq, Default, Debug)]
enum Stage {
    #[default]
    Idle,
    Loading,
    FieldSelect,
    Preview,
    Importing,
    Done,
    Error,
}

async fn read_file_as_bytes(file: &File) -> Result<Vec<u8>, String> {
    let reader = web_sys::FileReader::new().map_err(|e| format!("FileReader error: {:?}", e))?;
    let reader_clone = reader.clone();
    let (tx, rx) = futures::channel::oneshot::channel();
    let tx = Rc::new(RefCell::new(Some(tx)));

    let closure = wasm_bindgen::closure::Closure::<dyn FnMut()>::new(move || {
        if let Some(tx) = tx.borrow_mut().take() {
            let _ = tx.send(());
        }
    });
    if let Some(func) = closure.as_ref().dyn_ref::<Function>() {
        reader.set_onloadend(Some(func));
    }
    closure.forget();

    reader
        .read_as_array_buffer(file)
        .map_err(|e| format!("read_as_array_buffer error: {:?}", e))?;

    rx.await.map_err(|_| "File read cancelled".to_string())?;

    let result = reader_clone
        .result()
        .map_err(|e| format!("FileReader result error: {:?}", e))?;
    let array_buffer: js_sys::ArrayBuffer = result
        .dyn_into()
        .map_err(|_| "Result is not an ArrayBuffer".to_string())?;
    let uint8_array = js_sys::Uint8Array::new(&array_buffer);
    let mut bytes = vec![0u8; uint8_array.length() as usize];
    uint8_array.copy_to(&mut bytes);
    Ok(bytes)
}

#[component]
pub fn AnkiImportStage(
    is_open: RwSignal<bool>,
    refresh_trigger: RwSignal<u32>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let repository =
        use_context::<HybridUserRepository>().expect("repository context not provided");

    let stage = RwSignal::new(Stage::Idle);
    let error_message = RwSignal::new(String::new());
    let detected_fields = RwSignal::new(Vec::<AnkiFieldInfo>::new());
    let file_bytes = StoredValue::new(Vec::<u8>::new());
    let selected_word_field = RwSignal::new(String::new());
    let selected_translation_field = RwSignal::new(String::new());
    let extracted_cards = RwSignal::new(Vec::<AnkiCard>::new());
    let imported_count = RwSignal::new(0usize);
    let is_drag_over = RwSignal::new(false);
    let disposed = StoredValue::new(());

    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    Effect::new(move |_| {
        if !is_open.get() {
            stage.set(Stage::Idle);
            error_message.set(String::new());
            detected_fields.set(Vec::new());
            file_bytes.set_value(Vec::new());
            selected_word_field.set(String::new());
            selected_translation_field.set(String::new());
            extracted_cards.set(Vec::new());
            imported_count.set(0);
        }
    });

    let process_file = move |file: File| {
        if file.size() > MAX_APKG_SIZE_BYTES as f64 {
            stage.set(Stage::Error);
            error_message.set("Файл слишком большой (макс. 50 МБ)".to_string());
            return;
        }
        stage.set(Stage::Loading);
        let disposed = disposed;

        spawn_local(async move {
            let bytes = match read_file_as_bytes(&file).await {
                Ok(b) => b,
                Err(e) => {
                    if disposed.is_disposed() {
                        return;
                    }
                    stage.set(Stage::Error);
                    error_message.set(format!("Не удалось прочитать файл: {}", e));
                    return;
                },
            };
            if disposed.is_disposed() {
                return;
            }
            file_bytes.set_value(bytes.clone());

            match read_anki_database(&bytes) {
                Ok(deck_info) => {
                    if disposed.is_disposed() {
                        return;
                    }
                    if deck_info.detected_fields.is_empty() {
                        stage.set(Stage::Error);
                        error_message.set("Поля не найдены в колоде Anki".to_string());
                        return;
                    }
                    detected_fields.set(deck_info.detected_fields);
                    stage.set(Stage::FieldSelect);
                },
                Err(e) => {
                    if disposed.is_disposed() {
                        return;
                    }
                    stage.set(Stage::Error);
                    error_message.set(format!("Не удалось прочитать колоду: {}", e));
                },
            }
        });
    };

    let on_file_change = move |ev: leptos::ev::Event| {
        let target = match ev.target() {
            Some(t) => t,
            None => return,
        };
        let input: HtmlInputElement = match target.dyn_into() {
            Ok(i) => i,
            Err(_) => return,
        };
        let files = match input.files() {
            Some(f) => f,
            None => return,
        };
        if let Some(file) = files.get(0) {
            process_file(file);
        }
    };

    let on_drag_over = move |ev: leptos::ev::DragEvent| {
        ev.prevent_default();
        is_drag_over.set(true);
    };

    let on_drag_leave = move |ev: leptos::ev::DragEvent| {
        ev.prevent_default();
        is_drag_over.set(false);
    };

    let on_drop = move |ev: leptos::ev::DragEvent| {
        ev.prevent_default();
        is_drag_over.set(false);
        if let Some(dt) = ev.data_transfer()
            && let Some(files) = dt.files()
            && let Some(file) = files.get(0)
        {
            process_file(file);
        }
    };

    let field_options = Signal::derive(move || {
        detected_fields
            .get()
            .iter()
            .map(|f| DropdownItem {
                value: f.name.clone(),
                label: f.name.clone(),
            })
            .collect::<Vec<_>>()
    });

    let on_next = Callback::new(move |_: leptos::ev::MouseEvent| {
        let word_field = selected_word_field.get();
        if word_field.is_empty() {
            return;
        }
        let trans_field = selected_translation_field.get();
        let trans_opt = if trans_field.is_empty() {
            None
        } else {
            Some(trans_field)
        };
        let bytes = file_bytes.get_value();

        stage.set(Stage::Loading);

        spawn_local(async move {
            gloo_timers::future::sleep(std::time::Duration::from_millis(50)).await;

            if disposed.is_disposed() {
                return;
            }

            match extract_cards(&bytes, &word_field, trans_opt.as_deref()) {
                Ok((cards, _)) => {
                    if cards.is_empty() {
                        stage.set(Stage::Error);
                        error_message.set("Карточки не найдены".to_string());
                        return;
                    }
                    extracted_cards.set(cards);
                    stage.set(Stage::Preview);
                },
                Err(e) => {
                    stage.set(Stage::Error);
                    error_message.set(format!("Ошибка извлечения карточек: {}", e));
                },
            }
        });
    });

    let on_back = Callback::new(move |_: leptos::ev::MouseEvent| {
        stage.set(Stage::FieldSelect);
    });

    let on_import = {
        let repository = repository.clone();
        Callback::new(move |_: leptos::ev::MouseEvent| {
            let cards = extracted_cards.get();
            if cards.is_empty() {
                return;
            }
            stage.set(Stage::Importing);
            let repo = repository.clone();
            let refresh = refresh_trigger;
            let is_open_sig = is_open;
            let disposed = disposed;

            spawn_local(async move {
                let use_case = ImportAnkiPackUseCase::new(&repo);
                match use_case.execute(cards).await {
                    Ok(result) => {
                        if disposed.is_disposed() {
                            return;
                        }
                        imported_count.set(result.total_created_count);
                        stage.set(Stage::Done);
                        refresh.update(|v| *v += 1);
                        is_open_sig.set(false);
                    },
                    Err(e) => {
                        if disposed.is_disposed() {
                            return;
                        }
                        stage.set(Stage::Error);
                        error_message.set(format!("Ошибка импорта: {}", e));
                    },
                }
            });
        })
    };

    let on_retry = Callback::new(move |_: leptos::ev::MouseEvent| {
        stage.set(Stage::Idle);
        error_message.set(String::new());
        file_bytes.set_value(Vec::new());
    });

    view! {
        <div class="space-y-4" data-testid=test_id_val>
            {move || match stage.get() {
                Stage::Idle => view! {
                    <div
                        class=move || {
                            let base = "border-2 border-dashed rounded-lg p-8 text-center transition-colors cursor-pointer";
                            if is_drag_over.get() {
                                format!(
                                    "{} border-[var(--accent-olive)] bg-[var(--accent-olive)]/10",
                                    base
                                )
                            } else {
                                format!(
                                    "{} border-[var(--border-light)] hover:border-[var(--accent-olive)]/50",
                                    base
                                )
                            }
                        }
                        on:dragover=on_drag_over
                        on:dragleave=on_drag_leave
                        on:drop=on_drop
                        data-testid="anki-import-drop-zone"
                    >
                        <label class="cursor-pointer">
                            <input
                                type="file"
                                accept=".apkg,application/octet-stream"
                                class="hidden"
                                on:change=on_file_change
                                data-testid="anki-import-file-input"
                            />
                            <div class="space-y-2">
                                <svg
                                    class="mx-auto h-12 w-12 text-[var(--fg-muted)]"
                                    fill="none"
                                    viewBox="0 0 24 24"
                                    stroke="currentColor"
                                >
                                    <path
                                        stroke-linecap="round"
                                        stroke-linejoin="round"
                                        stroke-width="2"
                                        d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12"
                                    />
                                </svg>
                                <Text variant=TypographyVariant::Muted>
                                    "Перетащите .apkg файл или нажмите для выбора"
                                </Text>
                                <Text size=TextSize::Small variant=TypographyVariant::Muted>
                                    "Anki колода (макс. 50 МБ)"
                                </Text>
                            </div>
                        </label>
                    </div>
                }
                .into_any(),

                Stage::Loading => view! {
                    <div
                        class="flex flex-col items-center gap-3 py-8"
                        data-testid="anki-import-loading"
                    >
                        <Spinner />
                        <Text variant=TypographyVariant::Muted>"Обработка..."</Text>
                    </div>
                }
                .into_any(),

                Stage::FieldSelect => view! {
                    <div class="space-y-4">
                        <div>
                            <Text
                                size=TextSize::Small
                                variant=TypographyVariant::Muted
                                class=Signal::derive(|| "mb-2".to_string())
                            >
                                "Поле со словом"
                            </Text>
                            <Dropdown
                                options=field_options
                                selected=selected_word_field
                                placeholder=Signal::derive(|| "Выберите поле".to_string())
                                test_id=Signal::derive(|| "anki-import-field-word".to_string())
                            />
                        </div>
                        <div>
                            <Text
                                size=TextSize::Small
                                variant=TypographyVariant::Muted
                                class=Signal::derive(|| "mb-2".to_string())
                            >
                                "Поле с переводом (необязательно)"
                            </Text>
                            <Dropdown
                                options=field_options
                                selected=selected_translation_field
                                placeholder=Signal::derive(|| "Выберите поле".to_string())
                                test_id=Signal::derive(|| "anki-import-field-translation".to_string())
                            />
                        </div>
                        <div class="flex justify-end">
                            <Button
                                variant=ButtonVariant::Olive
                                disabled=Signal::derive(move || selected_word_field.get().is_empty())
                                on_click=on_next
                                test_id="anki-import-next-btn"
                            >
                                "Далее"
                            </Button>
                        </div>
                    </div>
                }
                .into_any(),

                Stage::Preview => view! {
                    <div class="space-y-4">
                        <div data-testid="anki-import-card-count">
                            <Text size=TextSize::Small variant=TypographyVariant::Muted>
                                {move || format!(
                                    "Найдено {} карточек",
                                    extracted_cards.get().len()
                                )}
                            </Text>
                        </div>
                        <div
                            class="space-y-2 overflow-y-auto max-h-64"
                            data-testid="anki-import-card-list"
                        >
                            <For
                                each=move || extracted_cards.get().clone()
                                key=|card| card.word.clone()
                                children=move |card| {
                                    let translation =
                                        card.translation.clone().unwrap_or_default();
                                    view! {
                                        <div class="flex justify-between items-center p-2 rounded bg-[var(--bg-secondary)]">
                                            <span class="font-medium">{card.word}</span>
                                            <span class="text-sm text-[var(--fg-muted)]">
                                                {translation}
                                            </span>
                                        </div>
                                    }
                                }
                            />
                        </div>
                        <div class="flex gap-2 justify-between">
                            <Button
                                variant=ButtonVariant::Ghost
                                on_click=on_back
                                test_id="anki-import-back-btn"
                            >
                                "Назад"
                            </Button>
                            <Button
                                variant=ButtonVariant::Olive
                                on_click=on_import
                                test_id="anki-import-import-btn"
                            >
                                "Импорт"
                            </Button>
                        </div>
                    </div>
                }
                .into_any(),

                Stage::Importing => view! {
                    <div class="flex flex-col items-center gap-3 py-8">
                        <Spinner />
                        <Text variant=TypographyVariant::Muted>"Импорт..."</Text>
                    </div>
                }
                .into_any(),

                Stage::Done => view! {
                    <div data-testid="anki-import-done">
                        <Alert
                            alert_type=AlertType::Success
                            title=Signal::derive(|| "Импорт завершён".to_string())
                            message=Signal::derive(move || {
                                format!("Импортировано {} карточек", imported_count.get())
                            })
                        />
                    </div>
                }
                .into_any(),

                Stage::Error => view! {
                    <div class="space-y-4" data-testid="anki-import-error">
                        <Alert
                            alert_type=AlertType::Error
                            title=Signal::derive(|| "Ошибка".to_string())
                            message=Signal::derive(move || error_message.get())
                        />
                        <Button
                            variant=ButtonVariant::Ghost
                            on_click=on_retry
                            test_id="anki-import-retry-btn"
                        >
                            "Попробовать снова"
                        </Button>
                    </div>
                }
                .into_any(),
            }}
        </div>
    }
}
