use crate::loaders::ModelLoader;
use crate::loaders::ocr_model_loader::ProgressCallback;
use crate::ui_components::{
    Alert, AlertType, Button, ButtonVariant, LoadingStageItem, OcrLoadingStage, OcrLoadingState,
    ProgressInfo, StageType, Text, TextSize, TypographyVariant, get_stage_info,
};
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::ocr::{JapaneseOCRModel, ModelConfig};
use origa::use_cases::ExtractTextFromImageUseCase;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use tracing::{debug, error, info};
use wasm_bindgen::JsCast;
use web_sys::js_sys::{Date, Function};
use web_sys::{ClipboardEvent, File, HtmlInputElement};

thread_local! {
    static CACHED_MODEL: RefCell<Option<Rc<RefCell<JapaneseOCRModel>>>> = const { RefCell::new(None) };
    static MODEL_LOADING: Cell<bool> = const { Cell::new(false) };
}

#[derive(Clone, Copy, Debug, Default)]
pub enum OcrState {
    #[default]
    Idle,
    Ready,
    Processing,
    Error,
}

const MAX_FILE_SIZE_MB: f64 = 10.0;
const MAX_BASE64_LEN: usize = 15_000_000;

#[derive(Clone)]
struct ProcessContext {
    image_preview: RwSignal<Option<String>>,
    ocr_state: RwSignal<OcrState>,
    ocr_loading_state: OcrLoadingState,
    error_message: RwSignal<Option<String>>,
}

fn handle_ocr_result(
    result: Result<String, String>,
    ctx: &ProcessContext,
    on_text_extracted: &Callback<String>,
) {
    match result {
        Ok(text) => {
            if text.trim().is_empty() {
                let err = "Не удалось распознать текст на изображении.";
                ctx.error_message.set(Some(err.to_string()));
                ctx.ocr_state.set(OcrState::Error);
                ctx.ocr_loading_state.stage.set(OcrLoadingStage::Error {
                    stage: "recognize".to_string(),
                    message: err.to_string(),
                });
            } else {
                info!(text_length = text.len(), "OCR completed successfully");
                ctx.ocr_state.set(OcrState::Ready);
                ctx.ocr_loading_state.stage.set(OcrLoadingStage::Completed);
                on_text_extracted.run(text);
            }
        }
        Err(e) => {
            error!(error = %e, "OCR failed");
            ctx.error_message.set(Some(e.clone()));
            ctx.ocr_state.set(OcrState::Error);
            ctx.ocr_loading_state.stage.set(OcrLoadingStage::Error {
                stage: "recognize".to_string(),
                message: e,
            });
        }
    }
}

async fn run_ocr_on_data_url(
    data_url: &str,
    ctx: &ProcessContext,
    on_text_extracted: &Callback<String>,
) {
    ctx.ocr_state.set(OcrState::Processing);
    ctx.ocr_loading_state.start_time.set(Some(Date::now()));

    let result = process_image_with_ocr(data_url, &ctx.ocr_loading_state).await;

    if ctx.ocr_loading_state.cancel_requested.get() {
        return;
    }

    handle_ocr_result(result, ctx, on_text_extracted);
}

fn process_file(
    file: File,
    ctx: ProcessContext,
    on_text_extracted: Callback<String>,
    on_error: Callback<String>,
    on_ocr_start: Callback<()>,
) {
    let file_name = file.name();
    debug!(file_name = %file_name, "File selected");

    if !is_image_file(&file) {
        ctx.error_message.set(Some(
            "Выберите изображение (PNG, JPEG или WebP)".to_string(),
        ));
        return;
    }

    let file_size_mb = file.size() / (1024.0 * 1024.0);
    if file_size_mb > MAX_FILE_SIZE_MB {
        ctx.error_message.set(Some(format!(
            "Файл слишком большой ({:.1} MB). Максимальный размер: {:.0} MB",
            file_size_mb, MAX_FILE_SIZE_MB
        )));
        return;
    }

    on_ocr_start.run(());

    spawn_local(async move {
        ctx.ocr_loading_state.cancel_requested.set(false);
        match read_file_as_data_url(&file).await {
            Ok(data_url) => {
                if ctx.ocr_loading_state.cancel_requested.get() {
                    return;
                }
                ctx.image_preview.set(Some(data_url.clone()));
                run_ocr_on_data_url(&data_url, &ctx, &on_text_extracted).await;
            }
            Err(e) => {
                error!(error = %e, "Failed to read file");
                ctx.error_message.set(Some(e.clone()));
                ctx.ocr_state.set(OcrState::Error);
                ctx.ocr_loading_state.stage.set(OcrLoadingStage::Error {
                    stage: "init".to_string(),
                    message: e.clone(),
                });
                on_error.run(e);
            }
        }
    });
}

#[component]
pub fn ImageInputStage(
    #[prop(optional, into)] class: Signal<String>,
    is_open: RwSignal<bool>,
    on_text_extracted: Callback<String>,
    on_error: Callback<String>,
    on_switch_to_text: Callback<()>,
    on_ocr_start: Callback<()>,
) -> impl IntoView {
    let ocr_state = RwSignal::new(OcrState::Idle);
    let image_preview = RwSignal::new(None::<String>);
    let error_message = RwSignal::new(None::<String>);
    let is_drag_over = RwSignal::new(false);
    let ocr_loading_state = OcrLoadingState::new();

    Effect::new(move |_| {
        if !is_open.get() {
            ocr_loading_state.cancel_requested.set(true);
            ocr_loading_state.reset();
            ocr_state.set(OcrState::Idle);
            image_preview.set(None);
            error_message.set(None);
        }
    });

    let ocr_loading_state_for_file = ocr_loading_state;
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
            process_file(
                file,
                ProcessContext {
                    image_preview,
                    ocr_state,
                    ocr_loading_state: ocr_loading_state_for_file,
                    error_message,
                },
                on_text_extracted,
                on_error,
                on_ocr_start,
            );
        }
    };

    let ocr_loading_state_for_drag = ocr_loading_state;
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

        if let Some(data_transfer) = ev.data_transfer()
            && let Some(files) = data_transfer.files()
            && let Some(file) = files.get(0)
        {
            process_file(
                file,
                ProcessContext {
                    image_preview,
                    ocr_state,
                    ocr_loading_state: ocr_loading_state_for_drag,
                    error_message,
                },
                on_text_extracted,
                on_error,
                on_ocr_start,
            );
        }
    };

    let ocr_loading_state_for_paste = ocr_loading_state;

    let stored_closure = StoredValue::new_local(None::<StoredClosure>);

    Effect::new(move |_| {
        let window = match web_sys::window() {
            Some(w) => w,
            None => return,
        };

        let ctx = ProcessContext {
            image_preview,
            ocr_state,
            ocr_loading_state: ocr_loading_state_for_paste,
            error_message,
        };

        let closure = wasm_bindgen::closure::Closure::<dyn FnMut(ClipboardEvent)>::new(
            move |event: ClipboardEvent| {
                if let Some(clipboard_data) = event.clipboard_data()
                    && let Some(files) = clipboard_data.files()
                    && let Some(file) = files.get(0)
                {
                    process_file(file, ctx.clone(), on_text_extracted, on_error, on_ocr_start);
                }
            },
        );

        let closure_ptr: Function = closure.as_ref().unchecked_ref::<Function>().clone();
        if window
            .add_event_listener_with_callback("paste", &closure_ptr)
            .is_ok()
        {
            stored_closure.set_value(Some(StoredClosure {
                window,
                closure_ptr,
                _closure: closure,
            }));
        }
    });

    let ocr_loading_state_for_cancel = ocr_loading_state;
    let ocr_state_for_cancel = ocr_state;
    let on_cancel = move |_| {
        ocr_loading_state_for_cancel.cancel_requested.set(true);
        ocr_state_for_cancel.set(OcrState::Idle);
    };

    let stage = ocr_loading_state.stage;

    view! {
        <div class=move || format!("{} space-y-4", class.get())>
            {move || {
                if matches!(ocr_state.get(), OcrState::Processing) {
                    view! {
                        <div class="space-y-4">
                            <h2 class="text-lg font-semibold text-[var(--fg-black)] flex items-center gap-2">
                                <span class="spinner spinner-sm"></span>
                                "Подготовка к распознаванию"
                            </h2>

                            <div class="space-y-3" role="list">
                                {move || {
                                    let info = get_stage_info(&stage.get(), StageType::Deim);
                                    view! {
                                        <LoadingStageItem
                                            status=info.status
                                            title="Сегментация текста".to_string()
                                            description=info.description
                                            progress=info.progress
                                            error_message=info.error_message
                                        />
                                    }
                                }}

                                {move || {
                                    let info = get_stage_info(&stage.get(), StageType::Parseq);
                                    view! {
                                        <LoadingStageItem
                                            status=info.status
                                            title="Распознавание символов".to_string()
                                            description=info.description
                                            progress=info.progress
                                            error_message=info.error_message
                                        />
                                    }
                                }}

                                {move || {
                                    let info = get_stage_info(&stage.get(), StageType::Init);
                                    view! {
                                        <LoadingStageItem
                                            status=info.status
                                            title="Инициализация моделей".to_string()
                                            description=info.description
                                            progress=info.progress
                                            error_message=info.error_message
                                        />
                                    }
                                }}

                                {move || {
                                    let info = get_stage_info(&stage.get(), StageType::Recognize);
                                    view! {
                                        <LoadingStageItem
                                            status=info.status
                                            title="Распознавание текста".to_string()
                                            description=info.description
                                            progress=info.progress
                                            error_message=info.error_message
                                        />
                                    }
                                }}
                            </div>

                            <div class="flex justify-end pt-2">
                                <Button
                                    variant=Signal::derive(|| ButtonVariant::Ghost)
                                    disabled=Signal::derive(move || ocr_loading_state.cancel_requested.get())
                                    on_click=Callback::new(on_cancel)
                                >
                                    {move || if ocr_loading_state.cancel_requested.get() { "Отмена..." } else { "Отменить" }}
                                </Button>
                            </div>
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <>
                            <div
                                class=move || {
                                    let base = "border-2 border-dashed rounded-lg p-8 text-center transition-colors cursor-pointer";
                                    if is_drag_over.get() {
                                        format!("{} border-[var(--accent-olive)] bg-[var(--accent-olive)]/10", base)
                                    } else {
                                        format!("{} border-[var(--border-light)] hover:border-[var(--accent-olive)]/50", base)
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
                                        <svg class="mx-auto h-12 w-12 text-[var(--fg-muted)]" fill="none" viewBox="0 0 24 24" stroke="currentColor">
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
                        </>
                    }.into_any()
                }
            }}
        </div>
    }
}

struct StoredClosure {
    window: web_sys::Window,
    closure_ptr: Function,
    _closure: wasm_bindgen::closure::Closure<dyn FnMut(ClipboardEvent)>,
}

impl Drop for StoredClosure {
    fn drop(&mut self) {
        let _ = self
            .window
            .remove_event_listener_with_callback("paste", &self.closure_ptr);
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

#[cfg(target_arch = "wasm32")]
async fn init_ocr_model(
    model_files: origa::ocr::ModelFiles,
    loading_state: &OcrLoadingState,
) -> Result<JapaneseOCRModel, String> {
    JapaneseOCRModel::from_model_files(model_files)
        .await
        .map_err(|e| {
            loading_state.stage.set(OcrLoadingStage::Error {
                stage: "init".to_string(),
                message: format!("Failed to initialize OCR model: {:?}", e),
            });
            format!("Failed to initialize OCR model: {:?}", e)
        })
}

#[cfg(not(target_arch = "wasm32"))]
async fn init_ocr_model(
    model_files: origa::ocr::ModelFiles,
    loading_state: &OcrLoadingState,
) -> Result<JapaneseOCRModel, String> {
    JapaneseOCRModel::from_model_files(model_files).map_err(|e| {
        loading_state.stage.set(OcrLoadingStage::Error {
            stage: "init".to_string(),
            message: format!("Failed to initialize OCR model: {:?}", e),
        });
        format!("Failed to initialize OCR model: {:?}", e)
    })
}

#[cfg(target_arch = "wasm32")]
async fn execute_ocr(
    use_case: &ExtractTextFromImageUseCase,
    model: &JapaneseOCRModel,
    bytes: &[u8],
) -> Result<String, String> {
    use_case
        .execute(model, bytes)
        .await
        .map_err(|e| format!("OCR failed: {:?}", e))
}

#[cfg(not(target_arch = "wasm32"))]
async fn execute_ocr(
    use_case: &ExtractTextFromImageUseCase,
    model: &JapaneseOCRModel,
    bytes: &[u8],
) -> Result<String, String> {
    use_case
        .execute(model, bytes)
        .map_err(|e| format!("OCR failed: {:?}", e))
}

#[allow(clippy::await_holding_refcell_ref)]
async fn process_image_with_ocr(
    data_url: &str,
    loading_state: &OcrLoadingState,
) -> Result<String, String> {
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
            if MODEL_LOADING.with(|loading| loading.get()) {
                return Err("OCR-модель уже загружается, пожалуйста подождите".to_string());
            }

            MODEL_LOADING.with(|loading| loading.set(true));
            debug!("Starting OCR model loading");

            let result = async {
                info!("Loading OCR and Layout models");

                let config =
                    ModelConfig::new(crate::core::config::ndlocr_base_url(), "ndlocr-model-");

                let loading_state_ref = *loading_state;
                let progress_callback: ProgressCallback =
                    Rc::new(move |filename, loaded, total| {
                        let stage = loading_state_ref.stage.get();
                        let start_time = loading_state_ref.start_time.get();

                        let percent = if total > 0 {
                            ((loaded as f64 / total as f64) * 100.0) as u32
                        } else {
                            0
                        };

                        let (speed_bps, eta_seconds) =
                            calculate_speed_and_eta(start_time, loaded, total);

                        let progress = ProgressInfo {
                            percent,
                            loaded_bytes: loaded,
                            total_bytes: total,
                            speed_bps,
                            eta_seconds,
                        };

                        if filename.contains("deim") {
                            loading_state_ref
                                .stage
                                .set(OcrLoadingStage::DownloadingDeim { progress });
                        } else if filename.contains("parseq") {
                            let current = match stage {
                                OcrLoadingStage::DownloadingParseq { current_model, .. } => {
                                    current_model
                                }
                                _ => 1,
                            };
                            loading_state_ref
                                .stage
                                .set(OcrLoadingStage::DownloadingParseq {
                                    current_model: current,
                                    progress,
                                });
                        }
                    });

                let loader = ModelLoader::new(config).with_progress_callback(progress_callback);

                loading_state.stage.set(OcrLoadingStage::DownloadingDeim {
                    progress: ProgressInfo::default(),
                });

                let model_files = loader.load_or_download_model().await.map_err(|e| {
                    loading_state.stage.set(OcrLoadingStage::Error {
                        stage: "deim".to_string(),
                        message: format!("Failed to load models: {:?}", e),
                    });
                    format!("Failed to load models: {:?}", e)
                })?;

                if loading_state.cancel_requested.get() {
                    return Err("Операция отменена".to_string());
                }

                loading_state.stage.set(OcrLoadingStage::Initializing {
                    model_name: "OCR models".to_string(),
                });

                let new_model = init_ocr_model(model_files, loading_state).await?;

                if loading_state.cancel_requested.get() {
                    return Err("Операция отменена".to_string());
                }

                let wrapped = Rc::new(RefCell::new(new_model));

                CACHED_MODEL.with(|cached| {
                    *cached.borrow_mut() = Some(wrapped.clone());
                });

                debug!("OCR model loaded and cached");
                Ok(wrapped)
            }
            .await;

            MODEL_LOADING.with(|loading| loading.set(false));

            result?
        }
    };

    if loading_state.cancel_requested.get() {
        return Err("Операция отменена".to_string());
    }

    loading_state.stage.set(OcrLoadingStage::Recognizing);

    info!("Running OCR with layout analysis");
    let use_case = ExtractTextFromImageUseCase::new();

    let model_ref = model.borrow();
    let result = execute_ocr(&use_case, &model_ref, &bytes).await;

    if loading_state.cancel_requested.get() {
        return Err("Операция отменена".to_string());
    }

    result
}

fn calculate_speed_and_eta(start_time: Option<f64>, loaded: u64, total: u64) -> (u64, u64) {
    let Some(start) = start_time else {
        return (0, 0);
    };

    let elapsed_ms = Date::now() - start;
    if elapsed_ms <= 0.0 {
        return (0, 0);
    }

    let elapsed_secs = elapsed_ms / 1000.0;
    let speed_bps = (loaded as f64 / elapsed_secs) as u64;

    if speed_bps == 0 || total == 0 || loaded >= total {
        return (speed_bps, 0);
    }

    let remaining_bytes = total - loaded;
    let eta_seconds = (remaining_bytes as f64 / speed_bps as f64) as u64;

    (speed_bps, eta_seconds)
}

fn base64_decode(input: &str) -> Result<Vec<u8>, String> {
    use base64::{Engine, engine::general_purpose::STANDARD};

    if input.len() > MAX_BASE64_LEN {
        return Err(format!(
            "Input too large: {} bytes (max {})",
            input.len(),
            MAX_BASE64_LEN
        ));
    }

    let input = input.replace(['\n', '\r'], "");

    if !input
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=')
    {
        return Err("Invalid base64 characters".to_string());
    }

    let padding = (4 - input.len() % 4) % 4;
    let padded_input = format!("{}{}", input, "=".repeat(padding));

    STANDARD
        .decode(&padded_input)
        .map_err(|e| format!("Base64 decode error: {}", e))
}
