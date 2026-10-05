//! Reusable audio-file speech-to-text pipeline.
//!
//! Extracted from `audio_input_stage` so the inbox (zero-tap intake) and the
//! drawer's Audio tab run the exact same validation and transcription path:
//! extension whitelist, WAV-only rule on the WASM fallback, 50 MB cap, then
//! native device-ai file ASR with the Whisper WASM fallback.

use leptos::prelude::*;
use leptos::task::spawn_local;
#[cfg(target_arch = "wasm32")]
use origa::stt::WhisperTranscriber;
#[cfg(target_arch = "wasm32")]
use std::cell::{Cell, RefCell};
#[cfg(target_arch = "wasm32")]
use std::rc::Rc;
#[cfg(target_arch = "wasm32")]
use tracing::{error, info};
use wasm_bindgen::JsCast;

#[cfg(target_arch = "wasm32")]
use crate::core::config::whisper_base_url;
#[cfg(target_arch = "wasm32")]
use crate::loaders::whisper_model_loader::WhisperModelLoader;
#[cfg(target_arch = "wasm32")]
use crate::utils::file::read_file_as_bytes;
#[cfg(target_arch = "wasm32")]
use base64::{Engine, engine::general_purpose::STANDARD};

/// Progress state of the audio transcription pipeline. The Audio tab and the
/// inbox both render their progress UI from this single source.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) enum AudioState {
    #[default]
    Idle,
    LoadingModel,
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    Processing,
    Ready,
    Error,
}

/// Reactive context a caller provides to [`transcribe_file`]. The Audio tab
/// and the inbox each pass their own signals, so progress renders in the
/// surface the user is looking at.
#[derive(Clone)]
pub(super) struct TranscribeContext {
    pub audio_state: RwSignal<AudioState>,
    pub status_text: RwSignal<Option<String>>,
    pub error_message: RwSignal<Option<String>>,
    pub disposed: StoredValue<()>,
    /// Run invalidation for surfaces that accept concurrent runs (the
    /// inbox): when the counter moves past `run_id`, this run is stale and
    /// every further state write and the final result are dropped. `None`
    /// for single-run surfaces (the Audio tab) keeps their behavior
    /// identical to the pre-extraction code.
    pub stale_run: Option<StaleRun>,
}

/// Invalidation token pairing the shared generation counter with the run id
/// this transcription was started under.
#[derive(Clone, Copy)]
pub(super) struct StaleRun {
    pub generation: RwSignal<u32>,
    pub run_id: u32,
}

impl TranscribeContext {
    /// True once this run has been superseded by a newer one or cancelled.
    pub fn is_stale(&self) -> bool {
        self.stale_run
            .as_ref()
            .is_some_and(|token| token.generation.get_untracked() != token.run_id)
    }
}

#[cfg(target_arch = "wasm32")]
thread_local! {
    static CACHED_WHISPER: RefCell<Option<Rc<WhisperTranscriber>>> = const { RefCell::new(None) };
    static WHISPER_LOADING: Cell<bool> = const { Cell::new(false) };
}

/// Stops a running Whisper model load. Exposed for cancel paths: the Audio
/// tab cancel button and the inbox cancel both flip this flag so a pending
/// load resolves instead of blocking the next attempt.
pub(super) fn cancel_whisper_loading() {
    #[cfg(target_arch = "wasm32")]
    WHISPER_LOADING.with(|l| l.set(false));
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

    let total_start = web_sys::js_sys::Date::now();
    let download_start = total_start;
    info!("Loading Whisper model from CDN");

    let loader = WhisperModelLoader::new(whisper_base_url());
    let files = loader
        .load()
        .await
        .map_err(|e| format!("Failed to download Whisper model: {:?}", e))?;

    let download_ms = web_sys::js_sys::Date::now() - download_start;
    info!(download_ms, "Whisper model files ready");

    status_text.set(Some("Initializing Whisper model...".to_string()));

    let init_start = web_sys::js_sys::Date::now();
    let model = WhisperModelLoader::init_model(files)
        .await
        .map_err(|e| format!("Failed to init Whisper model: {:?}", e))?;
    let init_ms = web_sys::js_sys::Date::now() - init_start;

    let wrapped = Rc::new(model);
    CACHED_WHISPER.with(|c| *c.borrow_mut() = Some(wrapped.clone()));
    let total_ms = web_sys::js_sys::Date::now() - total_start;
    info!(
        download_ms,
        init_ms, total_ms, "Whisper model loaded and cached"
    );
    Ok(wrapped)
}

#[cfg(target_arch = "wasm32")]
async fn transcribe_via_wasm(
    i18n: leptos_i18n::I18nContext<crate::i18n::Locale>,
    file: &web_sys::File,
    name: &str,
    ctx: &TranscribeContext,
) -> Result<String, String> {
    let bytes = read_file_as_bytes(file).await.map_err(|e| {
        error!(error = %e, "Audio file read failed");
        if !ctx.is_stale() {
            ctx.audio_state.set(AudioState::Error);
            ctx.error_message.set(Some(e.clone()));
        }
        e
    })?;

    // device-ai native file ASR is primary (no model download); Whisper WASM
    // is the fallback. Routing is runtime-resolved via capabilities, so on
    // Windows/Linux and the web the native path is unavailable and Whisper is
    // used transparently.
    if let Some(text) =
        super::asr_provider::recognize_file_via_device_ai(&STANDARD.encode(&bytes)).await
    {
        return Ok(text);
    }

    let model = get_or_load_whisper_model(ctx.status_text)
        .await
        .map_err(|e| {
            error!(error = %e, "Whisper model load failed");
            if !ctx.is_stale() {
                ctx.audio_state.set(AudioState::Error);
                ctx.error_message.set(Some(e.clone()));
            }
            e
        })?;

    // The i18n context is read off the reactive scope inside an async fn;
    // wrap in untrack to silence the reactive_graph warning without
    // pretending to subscribe.
    let loading_label = leptos::prelude::untrack(|| {
        i18n.get_keys()
            .words()
            .audio()
            .loading_model()
            .inner()
            .to_string()
    });
    if !ctx.is_stale() {
        ctx.status_text
            .set(Some(loading_label.replacen("{}", name, 1)));
        ctx.audio_state.set(AudioState::Processing);
    }

    let use_case = origa::use_cases::TranscribeAudioUseCase::new();
    let infer_start = web_sys::js_sys::Date::now();
    let result = use_case.execute(model.clone(), &bytes).await.map_err(|e| {
        error!(error = %e, "Whisper transcription failed");
        format!("Transcription failed: {:?}", e)
    });
    let infer_ms = web_sys::js_sys::Date::now() - infer_start;
    info!(
        infer_ms,
        bytes_len = bytes.len(),
        "Whisper inference timing"
    );
    result
}

#[cfg(target_arch = "wasm32")]
async fn dispatch_transcription(
    i18n: leptos_i18n::I18nContext<crate::i18n::Locale>,
    file: web_sys::File,
    name: String,
    ctx: TranscribeContext,
) -> Result<String, String> {
    transcribe_via_wasm(i18n, &file, &name, &ctx).await
}

#[cfg(not(target_arch = "wasm32"))]
async fn dispatch_transcription(
    _i18n: leptos_i18n::I18nContext<crate::i18n::Locale>,
    _file: web_sys::File,
    _name: String,
    _ctx: TranscribeContext,
) -> Result<String, String> {
    Err("Speech-to-text requires WASM runtime".to_string())
}

/// Validates and transcribes one audio file.
///
/// Rejects unsupported extensions, non-WAV files on the WASM fallback and
/// files over 50 MB *before* touching the pipeline state; every rejection
/// lands in `ctx.error_message` without flipping `audio_state`.
pub(super) fn transcribe_file(
    i18n: leptos_i18n::I18nContext<crate::i18n::Locale>,
    file: web_sys::File,
    ctx: TranscribeContext,
    on_text_extracted: Callback<String>,
    on_error: Callback<String>,
) {
    let name = file.name();
    let is_wav = name.ends_with(".wav");
    let valid_ext = is_wav
        || name.ends_with(".mp3")
        || name.ends_with(".webm")
        || name.ends_with(".m4a")
        || name.ends_with(".ogg");

    if !valid_ext {
        ctx.error_message.set(Some(
            i18n.get_keys()
                .words()
                .audio()
                .unsupported_format()
                .inner()
                .to_string(),
        ));
        return;
    }

    #[cfg(target_arch = "wasm32")]
    if !is_wav {
        ctx.error_message.set(Some(
            i18n.get_keys()
                .words()
                .audio()
                .wav_only()
                .inner()
                .to_string(),
        ));
        return;
    }

    let max_size_mb = 50.0;
    if file.size() / (1024.0 * 1024.0) > max_size_mb {
        ctx.error_message.set(Some(
            i18n.get_keys()
                .words()
                .audio()
                .file_too_large()
                .inner()
                .to_string()
                .replacen("{}", &max_size_mb.to_string(), 1),
        ));
        return;
    }

    ctx.audio_state.set(AudioState::LoadingModel);
    ctx.status_text.set(Some(
        i18n.get_keys()
            .words()
            .audio()
            .loading_model()
            .inner()
            .to_string()
            .replacen("{}", &name, 1),
    ));

    spawn_local(async move {
        let result = dispatch_transcription(i18n, file, name, ctx.clone()).await;

        // A superseded or cancelled run must touch neither the shared
        // state nor the callbacks — its result belongs to no visible run.
        if ctx.disposed.is_disposed() || ctx.is_stale() {
            return;
        }

        match result {
            Ok(text) => {
                if text.trim().is_empty() {
                    ctx.audio_state.set(AudioState::Error);
                    ctx.error_message.set(Some(
                        i18n.get_keys()
                            .words()
                            .audio()
                            .no_speech()
                            .inner()
                            .to_string(),
                    ));
                } else {
                    ctx.audio_state.set(AudioState::Ready);
                    ctx.status_text.set(None);
                    on_text_extracted.run(text);
                }
            },
            Err(e) => {
                ctx.audio_state.set(AudioState::Error);
                ctx.error_message.set(Some(e.clone()));
                on_error.run(e);
            },
        }
    });
}

/// Reads the selected file out of a file-input change event. Shared by the
/// Audio tab and any other `<input type="file">` consumer.
pub(super) fn file_from_change_event(ev: web_sys::Event) -> Option<web_sys::File> {
    let target = ev.target()?;
    let input: web_sys::HtmlInputElement = target.dyn_into().ok()?;
    let files = input.files()?;
    if files.length() > 0 {
        files.get(0)
    } else {
        None
    }
}
