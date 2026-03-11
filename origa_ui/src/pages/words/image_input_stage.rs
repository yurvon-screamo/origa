use crate::ocr_model_loader::ModelLoader;
use crate::ui_components::{
    Alert, AlertType, Button, ButtonVariant, Text, TextSize, TypographyVariant,
};
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::ocr::{JapaneseOCRModel, ModelConfig};
use origa::use_cases::ExtractTextFromImageUseCase;
use std::cell::RefCell;
use std::rc::Rc;
use tracing::{debug, error, info};
use wasm_bindgen::JsCast;
use web_sys::{ClipboardEvent, File, HtmlInputElement};

thread_local! {
    static CACHED_MODEL: Rc<RefCell<Option<Rc<RefCell<JapaneseOCRModel>>>>> = Rc::new(RefCell::new(None));
}

#[derive(Clone, Debug, Default)]
pub enum OcrState {
    #[default]
    Idle,
    Ready,
    Processing,
    Error,
}

const MAX_FILE_SIZE_MB: f64 = 10.0;

fn process_file(
    file: File,
    image_preview: RwSignal<Option<String>>,
    ocr_state: RwSignal<OcrState>,
    on_text_extracted: Callback<String>,
    on_error: Callback<String>,
    error_message: RwSignal<Option<String>>,
) {
    let file_name = file.name();
    debug!(file_name = %file_name, "File selected");

    if !is_image_file(&file) {
        error_message.set(Some(
            "Выберите изображение (PNG, JPEG или WebP)".to_string(),
        ));
        return;
    }

    let file_size_mb = file.size() / (1024.0 * 1024.0);
    if file_size_mb > MAX_FILE_SIZE_MB {
        error_message.set(Some(format!(
            "Файл слишком большой ({:.1} MB). Максимальный размер: {:.0} MB",
            file_size_mb, MAX_FILE_SIZE_MB
        )));
        return;
    }

    spawn_local(async move {
        ocr_state.set(OcrState::Processing);
        error_message.set(None);

        match read_file_as_data_url(&file).await {
            Ok(data_url) => {
                image_preview.set(Some(data_url.clone()));

                match process_image_with_ocr(&data_url).await {
                    Ok(text) => {
                        if text.trim().is_empty() {
                            let err = "Не удалось распознать текст на изображении. Попробуйте другое изображение или введите текст вручную.";
                            error_message.set(Some(err.to_string()));
                            ocr_state.set(OcrState::Error);
                            on_error.run(err.to_string());
                        } else {
                            info!(text_length = text.len(), "OCR completed successfully");
                            ocr_state.set(OcrState::Ready);
                            on_text_extracted.run(text);
                        }
                    }
                    Err(e) => {
                        error!(error = %e, "OCR failed");
                        error_message.set(Some(e.clone()));
                        ocr_state.set(OcrState::Error);
                        on_error.run(e);
                    }
                }
            }
            Err(e) => {
                error!(error = %e, "Failed to read file");
                error_message.set(Some(e.clone()));
                ocr_state.set(OcrState::Error);
                on_error.run(e);
            }
        }
    });
}

#[component]
pub fn ImageInputStage(
    #[prop(optional, into)] class: Signal<String>,
    on_text_extracted: Callback<String>,
    on_error: Callback<String>,
    on_switch_to_text: Callback<()>,
) -> impl IntoView {
    let ocr_state = RwSignal::new(OcrState::Idle);
    let image_preview = RwSignal::new(None::<String>);
    let error_message = RwSignal::new(None::<String>);
    let is_drag_over = RwSignal::new(false);

    let on_file_change = {
        let image_preview = image_preview;
        let ocr_state = ocr_state;
        let on_text_extracted = on_text_extracted;
        let on_error = on_error;
        let error_message = error_message;

        move |ev: leptos::ev::Event| {
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
                process_file(
                    file,
                    image_preview,
                    ocr_state,
                    on_text_extracted,
                    on_error,
                    error_message,
                );
            }
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

    let on_drop = {
        let image_preview = image_preview;
        let ocr_state = ocr_state;
        let on_text_extracted = on_text_extracted;
        let on_error = on_error;
        let error_message = error_message;

        move |ev: leptos::ev::DragEvent| {
            ev.prevent_default();
            is_drag_over.set(false);

            if let Some(data_transfer) = ev.data_transfer()
                && let Some(files) = data_transfer.files()
                && let Some(file) = files.get(0)
            {
                process_file(
                    file,
                    image_preview,
                    ocr_state,
                    on_text_extracted,
                    on_error,
                    error_message,
                );
            }
        }
    };

    Effect::new({
        let image_preview = image_preview;
        let ocr_state = ocr_state;
        let on_text_extracted = on_text_extracted;
        let on_error = on_error;
        let error_message = error_message;

        move || {
            let window = match web_sys::window() {
                Some(w) => w,
                None => return,
            };

            let closure = wasm_bindgen::closure::Closure::<dyn FnMut(ClipboardEvent)>::new({
                let image_preview = image_preview;
                let ocr_state = ocr_state;
                let on_text_extracted = on_text_extracted;
                let on_error = on_error;
                let error_message = error_message;

                move |event: ClipboardEvent| {
                    if let Some(clipboard_data) = event.clipboard_data()
                        && let Some(files) = clipboard_data.files()
                        && let Some(file) = files.get(0)
                    {
                        process_file(
                            file,
                            image_preview,
                            ocr_state,
                            on_text_extracted,
                            on_error,
                            error_message,
                        );
                    }
                }
            });

            let closure_ptr = closure.as_ref().unchecked_ref();
            if window
                .add_event_listener_with_callback("paste", closure_ptr)
                .is_ok()
            {
                closure.forget();
            }
        }
    });

    view! {
        <div class=move || format!("{} space-y-4", class.get())>
            <div
                class=move || {
                    let base = "border-2 border-dashed rounded-lg p-8 text-center transition-colors cursor-pointer";
                    if is_drag_over.get() {
                        format!("{} border-accent bg-accent/10", base)
                    } else {
                        format!("{} border-muted hover:border-accent/50", base)
                    }
                }
                on:dragover=on_drag_over
                on:dragleave=on_drag_leave
                on:drop=on_drop
            >
                <label class="cursor-pointer">
                    <input
                        type="file"
                        accept="image/png,image/jpeg,image/webp"
                        class="hidden"
                        on:change=on_file_change
                    />
                    <div class="space-y-2">
                        <svg class="mx-auto h-12 w-12 text-muted" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z" />
                        </svg>
                        <Text size=TextSize::Default variant=TypographyVariant::Muted>
                            "Перетащите изображение, вставьте из буфера обмена или нажмите для выбора"
                        </Text>
                        <Text size=TextSize::Small variant=TypographyVariant::Muted>
                            "PNG, JPEG, WebP (макс. 10 MB)"
                        </Text>
                    </div>
                </label>
            </div>

            {move || {
                image_preview.get().map(|src| view! {
                    <div class="relative">
                        <img src=src class="max-h-64 mx-auto rounded-lg shadow-md" alt="Preview" />
                    </div>
                })
            }}

            {move || {
                let state = ocr_state.get();
                match state {
                    OcrState::Processing => Some(view! {
                        <div class="flex items-center gap-2 text-muted">
                            <svg class="animate-spin h-4 w-4" fill="none" viewBox="0 0 24 24">
                                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                            </svg>
                            <Text size=TextSize::Small variant=TypographyVariant::Muted>
                                "Распознавание текста (при первом запуске загружаются модели OCR ~50MB и сегментации ~100MB)..."
                            </Text>
                        </div>
                    }),
                    _ => None,
                }
            }}

            {move || {
                error_message.get().map(|msg| view! {
                    <Alert
                        alert_type=Signal::derive(|| AlertType::Warning)
                        title=Signal::derive(|| "Не удалось распознать".to_string())
                        message=Signal::derive(move || msg.clone())
                    />
                    <Button
                        variant=ButtonVariant::Ghost
                        on_click=Callback::new(move |_| on_switch_to_text.run(()))
                    >
                        "Ввести текст вручную"
                    </Button>
                })
            }}
        </div>
    }
}

fn is_image_file(file: &File) -> bool {
    let mime = file.type_();
    mime == "image/png" || mime == "image/jpeg" || mime == "image/webp" || mime.is_empty()
}

async fn read_file_as_data_url(file: &File) -> Result<String, String> {
    let reader =
        web_sys::FileReader::new().map_err(|e| format!("Failed to create FileReader: {:?}", e))?;

    let reader_clone = reader.clone();
    let (sender, receiver) = futures::channel::oneshot::channel();

    let sender = Rc::new(RefCell::new(Some(sender)));
    let closure = wasm_bindgen::closure::Closure::<dyn FnMut()>::new(move || {
        if let Some(sender) = sender.borrow_mut().take() {
            let _ = sender.send(());
        }
    });

    reader.set_onloadend(Some(closure.as_ref().unchecked_ref()));
    closure.forget();

    reader
        .read_as_data_url(file)
        .map_err(|e| format!("Failed to read file: {:?}", e))?;

    receiver
        .await
        .map_err(|_| "Failed to wait for file read".to_string())?;

    let result = reader_clone
        .result()
        .map_err(|e| format!("Failed to get result: {:?}", e))?;
    result
        .as_string()
        .ok_or_else(|| "Result is not a string".to_string())
}

async fn process_image_with_ocr(data_url: &str) -> Result<String, String> {
    let base64_data = data_url
        .strip_prefix("data:image/")
        .and_then(|s| s.split_once(";base64,"))
        .map(|(_, data)| data)
        .ok_or("Invalid data URL format")?;

    let bytes = base64_decode(base64_data)?;

    let model = CACHED_MODEL.with(|cached| cached.borrow().clone());

    let model = match model {
        Some(m) => m,
        None => {
            info!("Loading OCR and Layout models");

            let config = ModelConfig::default();
            let loader = ModelLoader::new(config);

            let model_files = loader
                .load_or_download_model()
                .await
                .map_err(|e| format!("Failed to load models: {:?}", e))?;

            #[cfg(not(target_arch = "wasm32"))]
            let new_model = JapaneseOCRModel::from_model_files(model_files)
                .map_err(|e| format!("Failed to initialize OCR model: {:?}", e))?;

            #[cfg(target_arch = "wasm32")]
            let new_model = JapaneseOCRModel::from_model_files(model_files)
                .await
                .map_err(|e| format!("Failed to initialize OCR model: {:?}", e))?;

            let wrapped = Rc::new(RefCell::new(new_model));

            CACHED_MODEL.with(|cached| {
                *cached.borrow_mut() = Some(wrapped.clone());
            });

            wrapped
        }
    };

    info!("Running OCR with layout analysis");
    let use_case = ExtractTextFromImageUseCase::new();

    #[cfg(not(target_arch = "wasm32"))]
    let text = use_case
        .execute(&mut model.borrow_mut(), &bytes)
        .map_err(|e| format!("OCR failed: {:?}", e))?;

    #[cfg(target_arch = "wasm32")]
    let text = use_case
        .execute(&mut model.borrow_mut(), &bytes)
        .await
        .map_err(|e| format!("OCR failed: {:?}", e))?;

    Ok(text)
}

fn base64_decode(input: &str) -> Result<Vec<u8>, String> {
    use base64::{Engine, engine::general_purpose::STANDARD};

    let input = input.replace(['\n', '\r'], "");

    let padding = (4 - input.len() % 4) % 4;
    let padded_input = format!("{}{}", input, "=".repeat(padding));

    STANDARD
        .decode(&padded_input)
        .map_err(|e| format!("Base64 decode error: {}", e))
}
