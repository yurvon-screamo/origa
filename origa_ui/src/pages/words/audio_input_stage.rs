use leptos::prelude::*;
use leptos::task::spawn_local;
#[cfg(target_arch = "wasm32")]
use origa::stt::WhisperTranscriber;
#[cfg(target_arch = "wasm32")]
use std::cell::{Cell, RefCell};
#[cfg(target_arch = "wasm32")]
use std::rc::Rc;
#[cfg(target_arch = "wasm32")]
use tracing::info;
use wasm_bindgen::JsCast;

#[cfg(target_arch = "wasm32")]
use crate::core::config::whisper_base_url;
#[cfg(target_arch = "wasm32")]
use crate::loaders::whisper_model_loader::WhisperModelLoader;
use crate::ui_components::{Alert, AlertType, Button, ButtonVariant};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) enum AudioState {
    #[default]
    Idle,
    LoadingModel,
    #[allow(dead_code)]
    Processing,
    Ready,
    Error,
}

#[cfg(target_arch = "wasm32")]
thread_local! {
    static CACHED_WHISPER: RefCell<Option<Rc<WhisperTranscriber>>> = const { RefCell::new(None) };
    static WHISPER_LOADING: Cell<bool> = const { Cell::new(false) };
}

#[cfg(target_arch = "wasm32")]
async fn get_or_load_whisper_model(
    status_text: RwSignal<Option<String>>,
) -> Result<Rc<WhisperTranscriber>, String> {
    let cached = CACHED_WHISPER.with(|c| c.borrow().clone());
    if let Some(model) = cached {
        return Ok(model);
    }

    if WHISPER_LOADING.with(|l| l.get()) {
        return Err("Whisper model is already loading".to_string());
    }

    WHISPER_LOADING.with(|l| l.set(true));
    let result = load_whisper_model_inner(status_text).await;
    WHISPER_LOADING.with(|l| l.set(false));
    result
}

#[cfg(target_arch = "wasm32")]
async fn load_whisper_model_inner(
    status_text: RwSignal<Option<String>>,
) -> Result<Rc<WhisperTranscriber>, String> {
    status_text.set(Some("Downloading Whisper model...".to_string()));
    info!("Loading Whisper model from CDN");

    let loader = WhisperModelLoader::new(whisper_base_url());
    let files = loader
        .load()
        .await
        .map_err(|e| format!("Failed to download Whisper model: {:?}", e))?;

    status_text.set(Some("Initializing Whisper model...".to_string()));

    let model = WhisperModelLoader::init_model(files)
        .await
        .map_err(|e| format!("Failed to init Whisper model: {:?}", e))?;

    let wrapped = Rc::new(model);
    CACHED_WHISPER.with(|c| *c.borrow_mut() = Some(wrapped.clone()));
    info!("Whisper model loaded and cached");
    Ok(wrapped)
}

#[cfg(target_arch = "wasm32")]
async fn read_file_bytes(file: &web_sys::File) -> Result<Vec<u8>, String> {
    let array_buffer_promise = file.array_buffer();
    let array_buffer = wasm_bindgen_futures::JsFuture::from(array_buffer_promise)
        .await
        .map_err(|e| format!("Failed to read file data: {:?}", e))?;
    let uint8_array = js_sys::Uint8Array::new(&array_buffer);
    let mut bytes = vec![0u8; uint8_array.length() as usize];
    uint8_array.copy_to(&mut bytes);
    Ok(bytes)
}

#[cfg(target_arch = "wasm32")]
async fn transcribe_via_wasm(
    file: &web_sys::File,
    name: &str,
    status_text: RwSignal<Option<String>>,
    audio_state: RwSignal<AudioState>,
    error_message: RwSignal<Option<String>>,
) -> Result<String, String> {
    let model = get_or_load_whisper_model(status_text).await.map_err(|e| {
        audio_state.set(AudioState::Error);
        error_message.set(Some(e.clone()));
        e
    })?;

    status_text.set(Some(format!("Transcribing {}...", name)));
    audio_state.set(AudioState::Processing);

    let bytes = read_file_bytes(file).await.map_err(|e| {
        audio_state.set(AudioState::Error);
        error_message.set(Some(e.clone()));
        e
    })?;

    let use_case = origa::use_cases::TranscribeAudioUseCase::new();
    use_case
        .execute(model.clone(), &bytes)
        .await
        .map_err(|e| format!("Transcription failed: {:?}", e))
}

#[cfg(target_arch = "wasm32")]
fn dispatch_transcription(
    file: &web_sys::File,
    name: &str,
    audio_state_local: RwSignal<AudioState>,
    status_text_local: RwSignal<Option<String>>,
    error_message_local: RwSignal<Option<String>>,
) -> impl std::future::Future<Output = Result<String, String>> {
    let file = file.clone();
    let name = name.to_string();
    async move {
        transcribe_via_wasm(
            &file,
            &name,
            status_text_local,
            audio_state_local,
            error_message_local,
        )
        .await
    }
}

#[cfg(not(target_arch = "wasm32"))]
async fn dispatch_transcription(
    _file: &web_sys::File,
    _name: &str,
    _audio_state_local: RwSignal<AudioState>,
    _status_text_local: RwSignal<Option<String>>,
    _error_message_local: RwSignal<Option<String>>,
) -> Result<String, String> {
    Err("Speech-to-text requires WASM runtime".to_string())
}

#[component]
pub(super) fn AudioInputStage(
    is_open: Signal<bool>,
    on_text_extracted: Callback<String>,
    on_error: Callback<String>,
    on_switch_to_text: Callback<()>,
) -> impl IntoView {
    let audio_state = RwSignal::new(AudioState::Idle);
    let error_message = RwSignal::new(None::<String>);
    let status_text = RwSignal::new(None::<String>);

    Effect::new(move |_| {
        if !is_open.get() {
            audio_state.set(AudioState::Idle);
            error_message.set(None);
            status_text.set(None);
        }
    });

    let handle_file = move |file: web_sys::File| {
        let name = file.name();
        let is_wav = name.ends_with(".wav");
        let valid_ext = is_wav
            || name.ends_with(".mp3")
            || name.ends_with(".webm")
            || name.ends_with(".m4a")
            || name.ends_with(".ogg");

        if !valid_ext {
            error_message.set(Some(
                "Unsupported audio format. Please use WAV, MP3, WebM, M4A, or OGG.".to_string(),
            ));
            return;
        }

        #[cfg(target_arch = "wasm32")]
        if !is_wav {
            error_message.set(Some(
                "Only WAV format is supported for speech recognition.".to_string(),
            ));
            return;
        }

        let max_size_mb = 50.0;
        if file.size() / (1024.0 * 1024.0) > max_size_mb {
            error_message.set(Some(format!(
                "Audio file is too large. Maximum size is {} MB.",
                max_size_mb
            )));
            return;
        }

        audio_state.set(AudioState::LoadingModel);
        status_text.set(Some(format!("Loading model for {}...", name)));

        let on_text_extracted = on_text_extracted;
        let on_error = on_error;
        let audio_state_local = audio_state;
        let status_text_local = status_text;
        let error_message_local = error_message;

        spawn_local(async move {
            let result = dispatch_transcription(
                &file,
                &name,
                audio_state_local,
                status_text_local,
                error_message_local,
            )
            .await;

            match result {
                Ok(text) => {
                    if text.trim().is_empty() {
                        audio_state_local.set(AudioState::Error);
                        error_message_local
                            .set(Some("No speech detected in the audio.".to_string()));
                    } else {
                        audio_state_local.set(AudioState::Ready);
                        status_text_local.set(None);
                        on_text_extracted.run(text);
                    }
                },
                Err(e) => {
                    audio_state_local.set(AudioState::Error);
                    error_message_local.set(Some(e.clone()));
                    on_error.run(e);
                },
            }
        });
    };

    let on_change = move |ev: web_sys::Event| {
        let target = match ev.target() {
            Some(t) => t,
            None => return,
        };
        let input: web_sys::HtmlInputElement = match target.dyn_into() {
            Ok(i) => i,
            Err(_) => return,
        };
        let files = match input.files() {
            Some(f) => f,
            None => return,
        };
        if files.length() > 0 {
            if let Some(file) = files.get(0) {
                handle_file(file);
            }
        }
    };

    view! {
        <div class="space-y-4">
            {move || {
                match audio_state.get() {
                    AudioState::LoadingModel | AudioState::Processing => view! {
                        <div class="space-y-4">
                            <div class="text-lg font-semibold text-[var(--fg-black)] flex items-center gap-2">
                                <span class="spinner spinner-sm"></span>
                                {move || status_text.get().unwrap_or_else(|| "Processing audio...".to_string())}
                            </div>
                            <Button
                                variant=Signal::derive(|| ButtonVariant::Ghost)
                                on_click=Callback::new(move |_| {
                                    audio_state.set(AudioState::Idle);
                                    #[cfg(target_arch = "wasm32")]
                                    WHISPER_LOADING.with(|l| l.set(false));
                                })
                            >
                                "Cancel"
                            </Button>
                        </div>
                    }.into_any(),
                    _ => view! {
                        <>
                            <div class="border-2 border-dashed rounded-lg p-8 text-center transition-colors cursor-pointer border-[var(--border-light)] hover:border-[var(--accent-olive)]/50">
                                <label class="cursor-pointer">
                                    <input
                                        type="file"
                                        accept="audio/*"
                                        class="hidden"
                                        on:change=on_change
                                    />
                                    <div class="space-y-2">
                                        <svg class="mx-auto h-12 w-12 text-[var(--fg-muted)]" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11a7 7 0 01-7 7m0 0a7 7 0 017-7m-7 7h18m-18 0a7 7 0 017 7m0 0a7 7 0 01-7-7m-7 7h18" />
                                        </svg>
                                        <p class="text-sm text-[var(--fg-muted)]">Click or drag to upload audio</p>
                                        <p class="text-xs text-[var(--fg-muted)]">WAV (max 50 MB)</p>
                                    </div>
                                </label>
                            </div>
                            {move || {
                                error_message.get().map(|msg| view! {
                                    <div>
                                        <Alert
                                            alert_type=Signal::derive(|| AlertType::Warning)
                                            title=Signal::derive(|| "Transcription failed".to_string())
                                            message=Signal::derive(move || msg.clone())
                                        />
                                        <Button
                                            variant=ButtonVariant::Ghost
                                            on_click=Callback::new(move |_| on_switch_to_text.run(()))
                                        >
                                            "Enter text manually"
                                        </Button>
                                    </div>
                                })
                            }}
                        </>
                    }.into_any(),
                }
            }}
        </div>
    }
}
