//! Audio speech-to-text pipeline shared by the Audio tab and the inbox.
//!
//! `transcribe_file` is the single entry point: it validates the file size,
//! streams the audio through [`super::audio_chunks::ChunkedDecoder`] into
//! 30-second WAV chunks, and transcribes each chunk via native device-ai
//! with the Whisper WASM fallback. Chunking removes the old wav-only /
//! 50 MB / single-shot limits: format validation lives entirely in the
//! symphonia probe (any decodable container of any length works), and a
//! cancelled/superseded run is fenced by the stale-run token.

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
use tracing::{error, warn};
use wasm_bindgen::JsCast;

#[cfg(target_arch = "wasm32")]
use crate::core::config::whisper_base_url;
#[cfg(target_arch = "wasm32")]
use crate::loaders::whisper_model_loader::WhisperModelLoader;
use crate::utils::file::read_file_as_bytes;
#[cfg(target_arch = "wasm32")]
use base64::{Engine, engine::general_purpose::STANDARD};

use super::audio_chunks::{ChunkedDecoder, WavChunk, join_chunk_texts};
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
pub(super) async fn transcribe_wav_bytes(
    wav_bytes: &[u8],
    ctx: &TranscribeContext,
) -> Result<String, String> {
    // device-ai native file ASR is primary (no model download); Whisper WASM
    // is the fallback. Routing is runtime-resolved via capabilities, so on
    // Windows/Linux and the web the native path is unavailable and Whisper is
    // used transparently.
    if let Some(text) =
        super::asr_provider::recognize_file_via_device_ai(&STANDARD.encode(wav_bytes)).await
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

    let use_case = origa::use_cases::TranscribeAudioUseCase::new();
    let infer_start = web_sys::js_sys::Date::now();
    let result = use_case
        .execute(model.clone(), wav_bytes)
        .await
        .map_err(|e| {
            error!(error = %e, "Whisper transcription failed");
            format!("Transcription failed: {:?}", e)
        });
    let infer_ms = web_sys::js_sys::Date::now() - infer_start;
    info!(
        infer_ms,
        bytes_len = wav_bytes.len(),
        "Whisper inference timing"
    );
    result
}

#[cfg(not(target_arch = "wasm32"))]
pub(super) async fn transcribe_wav_bytes(
    _wav_bytes: &[u8],
    _ctx: &TranscribeContext,
) -> Result<String, String> {
    Err("Speech-to-text requires WASM runtime".to_string())
}

/// Upper file-size bound. Chunked decoding keeps peak memory near
/// `input bytes + one chunk`, so the cap guards memory, not duration.
const MAX_FILE_SIZE_MB: f64 = 200.0;

/// Validates and transcribes one audio file of any supported format and
/// length.
///
/// Format validation lives entirely in the symphonia probe — a broken or
/// unsupported file surfaces as a decode error without touching the pipeline
/// state. Transcription runs per 30-second chunk: empty chunks (silence/
/// music) are skipped, and a chunk error aborts with the fragment number
/// while partial text is intentionally not delivered (the error plus the
/// manual-input fallback is the recovery path).
pub(super) fn transcribe_file(
    i18n: leptos_i18n::I18nContext<crate::i18n::Locale>,
    file: web_sys::File,
    ctx: TranscribeContext,
    on_text_extracted: Callback<String>,
    on_error: Callback<String>,
) {
    let name = file.name();
    if file.size() / (1024.0 * 1024.0) > MAX_FILE_SIZE_MB {
        ctx.error_message.set(Some(
            i18n.get_keys()
                .words()
                .audio()
                .file_too_large()
                .inner()
                .to_string()
                .replacen("{}", &MAX_FILE_SIZE_MB.to_string(), 1),
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
        let bytes = match read_file_as_bytes(&file).await {
            Ok(bytes) => bytes,
            Err(e) => {
                error!(error = %e, "Audio file read failed");
                if !ctx.disposed.is_disposed() && !ctx.is_stale() {
                    ctx.audio_state.set(AudioState::Error);
                    ctx.error_message.set(Some(e.clone()));
                    on_error.run(e);
                }
                return;
            },
        };

        let mut decoder = match ChunkedDecoder::new(bytes) {
            Ok(decoder) => decoder,
            Err(e) => {
                warn!(error = %e, "Audio decode failed");
                if !ctx.disposed.is_disposed() && !ctx.is_stale() {
                    ctx.audio_state.set(AudioState::Error);
                    ctx.error_message.set(Some(e));
                    // No details to surface beyond the probe message — the
                    // manual-input fallback covers the recovery path.
                    on_error.run(String::new());
                }
                return;
            },
        };

        let ctx_for_loop = ctx.clone();
        let joined = match run_chunk_loop(
            || decoder.next_chunk(),
            || ctx.disposed.is_disposed() || ctx.is_stale(),
            |fragment| {
                if !ctx.is_stale() {
                    ctx.status_text.set(Some(
                        i18n.get_keys()
                            .words()
                            .audio()
                            .fragment_progress()
                            .inner()
                            .to_string()
                            .replacen("{}", &fragment.to_string(), 1),
                    ));
                    ctx.audio_state.set(AudioState::Processing);
                }
            },
            |wav_bytes| {
                let ctx = ctx_for_loop.clone();
                async move { transcribe_wav_bytes(&wav_bytes, &ctx).await }
            },
        )
        .await
        {
            Ok(text) => text,
            Err((fragment, reason)) => {
                let message = i18n
                    .get_keys()
                    .words()
                    .audio()
                    .interrupted_on_fragment()
                    .inner()
                    .to_string()
                    .replacen("{}", &fragment.to_string(), 1)
                    .replacen("{}", &reason, 1);
                error!(fragment, error = %reason, "Chunked transcription aborted");
                if !ctx.disposed.is_disposed() && !ctx.is_stale() {
                    ctx.audio_state.set(AudioState::Error);
                    ctx.error_message.set(Some(message.clone()));
                    on_error.run(message);
                }
                return;
            },
        };

        if ctx.disposed.is_disposed() || ctx.is_stale() {
            return;
        }

        if joined.trim().is_empty() {
            ctx.audio_state.set(AudioState::Error);
            ctx.error_message.set(Some(
                i18n.get_keys()
                    .words()
                    .audio()
                    .no_speech()
                    .inner()
                    .to_string(),
            ));
            return;
        }

        ctx.audio_state.set(AudioState::Ready);
        ctx.status_text.set(None);
        on_text_extracted.run(joined);
    });
}

/// The chunk loop, separated from UI wiring for testability.
///
/// Returns the joined transcription, or `(fragment_number, reason)` when a
/// chunk fails. Silent chunks are skipped; a cancelled run yields the
/// accumulated text, which the caller discards via its own stale check.
pub(super) async fn run_chunk_loop<D, C, F, Fut, P>(
    mut next_chunk: D,
    is_cancelled: C,
    mut on_fragment: P,
    mut recognize: F,
) -> Result<String, (u32, String)>
where
    D: FnMut() -> Option<Result<WavChunk, String>>,
    C: Fn() -> bool,
    P: FnMut(u32),
    F: FnMut(Vec<u8>) -> Fut,
    Fut: std::future::Future<Output = Result<String, String>>,
{
    let mut texts: Vec<String> = Vec::new();
    let mut fragment = 0_u32;
    loop {
        if is_cancelled() {
            return Ok(join_chunk_texts(&texts));
        }
        let Some(chunk_result) = next_chunk() else {
            break;
        };
        let chunk = match chunk_result {
            Ok(chunk) => chunk,
            Err(reason) => return Err((fragment + 1, reason)),
        };

        fragment += 1;
        on_fragment(fragment);

        match recognize(chunk.wav_bytes).await {
            Ok(text) if text.trim().is_empty() => {}, // silent chunk — skip
            Ok(text) => texts.push(text),
            Err(reason) => return Err((fragment, reason)),
        }
    }
    Ok(join_chunk_texts(&texts))
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

#[cfg(test)]
mod chunk_loop_tests {
    use super::*;
    use std::cell::RefCell;

    use rstest::rstest;

    fn wav_chunk() -> WavChunk {
        WavChunk {
            wav_bytes: encode_wav_placeholder(),
        }
    }

    /// Minimal stand-in payload: run_chunk_loop is agnostic to the bytes.
    fn encode_wav_placeholder() -> Vec<u8> {
        vec![0u8; 64]
    }

    /// Drives the loop with a scripted chunk source and a scripted
    /// recognizer; returns (joined_text, fragments, recognized_payloads).
    async fn run_scripted(
        chunks: Vec<Result<WavChunk, String>>,
        cancelled: bool,
        recognize_results: Vec<Result<String, String>>,
    ) -> (Result<String, (u32, String)>, Vec<u32>, Vec<Vec<u8>>) {
        let queue = RefCell::new(chunks.into_iter());
        let recognized = RefCell::new(recognize_results.into_iter());
        let fragments = RefCell::new(Vec::new());
        let payloads = RefCell::new(Vec::new());

        let result = run_chunk_loop(
            || queue.borrow_mut().next(),
            || cancelled,
            |fragment| fragments.borrow_mut().push(fragment),
            |wav_bytes| {
                payloads.borrow_mut().push(wav_bytes);
                let next = recognized
                    .borrow_mut()
                    .next()
                    .expect("scripted recognizer exhausted");
                async move { next }
            },
        )
        .await;
        (result, fragments.into_inner(), payloads.into_inner())
    }

    #[tokio::test]
    async fn multiple_chunks_join_in_order() {
        let chunks = vec![Ok(wav_chunk()), Ok(wav_chunk()), Ok(wav_chunk())];
        let results = vec![
            Ok("私は".to_string()),
            Ok("本を".to_string()),
            Ok("読みます".to_string()),
        ];
        let (result, fragments, payloads) = run_scripted(chunks, false, results).await;
        assert_eq!(result.expect("must succeed"), "私は本を読みます");
        assert_eq!(fragments, vec![1, 2, 3]);
        assert_eq!(payloads.len(), 3);
    }

    #[tokio::test]
    async fn silent_middle_chunk_is_skipped_without_failure() {
        let chunks = vec![Ok(wav_chunk()), Ok(wav_chunk()), Ok(wav_chunk())];
        let results = vec![
            Ok("私は".to_string()),
            Ok("   ".to_string()), // silence/music — skipped
            Ok("読みます".to_string()),
        ];
        let (result, fragments, payloads) = run_scripted(chunks, false, results).await;
        assert_eq!(result.expect("must succeed"), "私は読みます");
        assert_eq!(fragments, vec![1, 2, 3]); // fragment counted, text skipped
        assert_eq!(payloads.len(), 3);
    }

    #[tokio::test]
    async fn recognize_failure_aborts_with_the_fragment_number() {
        let chunks = vec![Ok(wav_chunk()), Ok(wav_chunk()), Ok(wav_chunk())];
        let results = vec![
            Ok("私は".to_string()),
            Err("inference blew up".to_string()),
            Ok("unreachable".to_string()),
        ];
        let (result, fragments, payloads) = run_scripted(chunks, false, results).await;
        assert_eq!(
            result.expect_err("must fail fast"),
            (2, "inference blew up".to_string())
        );
        assert_eq!(fragments, vec![1, 2]);
        assert_eq!(payloads.len(), 2); // fail-fast: chunk 3 never recognized
    }

    #[tokio::test]
    async fn decode_failure_reports_the_incoming_fragment() {
        let chunks = vec![
            Ok(wav_chunk()),
            Err("corrupt container".to_string()),
            Ok(wav_chunk()),
        ];
        let results = vec![Ok("私は".to_string())];
        let (result, fragments, payloads) = run_scripted(chunks, false, results).await;
        // The failed chunk is the one about to be transcribed (fragment 2).
        assert_eq!(
            result.expect_err("must fail fast"),
            (2, "corrupt container".to_string())
        );
        assert_eq!(fragments, vec![1]);
        assert_eq!(payloads.len(), 1);
    }

    #[tokio::test]
    async fn cancellation_yields_accumulated_text_for_the_caller_to_discard() {
        let chunks = vec![Ok(wav_chunk()), Ok(wav_chunk())];
        let results = vec![Ok("私は".to_string()), Ok("本を".to_string())];
        let (result, fragments, _payloads) = run_scripted(chunks, true, results).await;
        // The caller drops this via its own stale check — the contract is
        // "no error, no state writes", not "nothing returned".
        assert_eq!(result.expect("cancel is not an error"), "");
        assert!(fragments.is_empty());
    }

    #[rstest]
    #[case::all_silent(vec!["", "  "], "")]
    #[case::single_chunk(vec!["本"], "本")]
    fn joined_text_rules_hold_for_edge_inputs(
        #[case] recognized: Vec<&str>,
        #[case] expected: &str,
    ) {
        let texts: Vec<String> = recognized.into_iter().map(String::from).collect();
        assert_eq!(join_chunk_texts(&texts), expected);
    }
}
