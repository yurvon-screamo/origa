use crate::ocr_model_loader::ModelLoader;
use crate::ui_components::{
    Alert, AlertType, Button, ButtonVariant, Text, TextSize, TypographyVariant,
};
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::ocr::{JapaneseOCRModel, ModelConfig};
use std::cell::RefCell;
use std::rc::Rc;
use tracing::{debug, error, info};
use wasm_bindgen::JsCast;
use web_sys::{File, HtmlInputElement};

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

    let on_file_change = move |ev: leptos::ev::Event| {
        let input: HtmlInputElement = ev.target().unwrap().dyn_into().unwrap();
        let files = match input.files() {
            Some(f) => f,
            None => return,
        };

        if let Some(file) = files.get(0) {
            let file_name = file.name();
            debug!(file_name = %file_name, "File selected");

            if !is_image_file(&file) {
                error_message.set(Some(
                    "Пожалуйста, выберите изображение (PNG, JPEG, WebP)".to_string(),
                ));
                return;
            }

            let file_clone = file.clone();
            let image_preview_clone = image_preview;
            let ocr_state_clone = ocr_state;
            let on_text_extracted_clone = on_text_extracted;
            let on_error_clone = on_error;
            let error_message_clone = error_message;

            spawn_local(async move {
                ocr_state_clone.set(OcrState::Processing);
                error_message_clone.set(None);

                match read_file_as_data_url(&file_clone).await {
                    Ok(data_url) => {
                        image_preview_clone.set(Some(data_url.clone()));

                        match process_image_with_ocr(&data_url).await {
                            Ok(text) => {
                                if text.trim().is_empty() {
                                    let err = "Не удалось распознать текст на изображении. Попробуйте другое изображение или введите текст вручную.";
                                    error_message_clone.set(Some(err.to_string()));
                                    ocr_state_clone.set(OcrState::Error);
                                    on_error_clone.run(err.to_string());
                                } else {
                                    info!(text_length = text.len(), "OCR completed successfully");
                                    ocr_state_clone.set(OcrState::Ready);
                                    on_text_extracted_clone.run(text);
                                }
                            }
                            Err(e) => {
                                error!(error = %e, "OCR failed");
                                error_message_clone.set(Some(e.clone()));
                                ocr_state_clone.set(OcrState::Error);
                                on_error_clone.run(e);
                            }
                        }
                    }
                    Err(e) => {
                        error!(error = %e, "Failed to read file");
                        error_message_clone.set(Some(e.clone()));
                        ocr_state_clone.set(OcrState::Error);
                        on_error_clone.run(e);
                    }
                }
            });
        }
    };

    view! {
        <div class=move || format!("space-y-4 {}", class.get())>
            <div>
                <Text size=TextSize::Small variant=TypographyVariant::Muted class=Signal::derive(|| "mb-2".to_string())>
                    "Загрузите изображение с японским текстом"
                </Text>
            </div>

            <div class="flex flex-col items-center justify-center w-full">
                <label class="flex flex-col items-center justify-center w-full h-32 border-2 border-dashed rounded-lg cursor-pointer bg-hover hover:bg-hover-dark border-border transition-colors">
                    <div class="flex flex-col items-center justify-center pt-5 pb-6">
                        <svg class="w-8 h-8 mb-2 text-muted" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12"></path>
                        </svg>
                        <p class="mb-2 text-sm text-muted">
                            <span class="font-semibold">"Нажмите для загрузки"</span>
                        </p>
                        <p class="text-xs text-muted">"PNG, JPEG или WebP"</p>
                    </div>
                    <input
                        type="file"
                        class="hidden"
                        accept="image/png,image/jpeg,image/webp"
                        on:change=on_file_change
                        disabled=Signal::derive(move || matches!(ocr_state.get(), OcrState::Processing))
                    />
                </label>
            </div>

            {move || {
                image_preview.get().map(|src| view! {
                    <div class="relative">
                        <img
                            src=src
                            alt="Preview"
                            class="w-full max-h-48 object-contain rounded-lg border border-border"
                        />
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
                                "Распознавание текста (при первом запуске загружается модель OCR ~50MB)..."
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

    let img =
        image::load_from_memory(&bytes).map_err(|e| format!("Failed to decode image: {}", e))?;

    let model = CACHED_MODEL.with(|cached| cached.borrow().clone());

    let model = match model {
        Some(m) => m,
        None => {
            let loader = ModelLoader::new(ModelConfig::new(
                "https://huggingface.co",
                "l0wgear/manga-ocr-2025-onnx",
                ".manga-ocr",
            ));
            let model_files = loader
                .load_or_download_model()
                .await
                .map_err(|e| format!("Failed to load OCR model: {:?}", e))?;

            let new_model = JapaneseOCRModel::from_model_files(model_files)
                .map_err(|e| format!("Failed to initialize OCR model: {:?}", e))?;

            let wrapped = Rc::new(RefCell::new(new_model));

            CACHED_MODEL.with(|cached| {
                *cached.borrow_mut() = Some(wrapped.clone());
            });

            wrapped
        }
    };

    let text = model
        .borrow_mut()
        .run(&img)
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
