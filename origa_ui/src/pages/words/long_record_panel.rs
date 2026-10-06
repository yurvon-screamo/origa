// Long-form recording panel (AU-3): arbitrary-length microphone capture
// that hands recorded WAV chunks to the chunked transcription pipeline
// and lands the result on the transcript screen. Browser-only module
// (mic capture through the WebAudio graph).
//
// WebAudio captures mono PCM at a 16 kHz AudioContext (the browser
// resamples the mic input); 30-second chunks are encoded to WAV while
// recording, so memory grows ~1 MB per chunk. MediaRecorder is
// deliberately not used — its webm/opus output has no decoder in the
// stack.
//
// Capture state lives in thread_locals (the pattern of the cached Whisper
// model): the UI callbacks are Send-gated, and Rc handles are not.
#[cfg(target_arch = "wasm32")]
use leptos::prelude::*;
use leptos::task::spawn_local;
use std::cell::RefCell;
#[cfg(all(target_arch = "wasm32", feature = "wasm-test"))]
use tracing::info;
use tracing::warn;
use wasm_bindgen::{JsCast, JsValue};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::JsFuture;

use super::audio_chunks::encode_wav_s16_mono;
use super::audio_chunks::join_chunk_texts;
use super::audio_transcribe::{AudioState, TranscribeContext, transcribe_wav_bytes};
use crate::i18n::{t, use_i18n};
use crate::ui_components::{Button, ButtonVariant, Text, TextSize, TypographyVariant};
#[cfg(not(target_arch = "wasm32"))]
use tracing::info;

const CHUNK_SECONDS: f64 = 30.0;
const TARGET_RATE: u32 = 16_000;

thread_local! {
    static ACTIVE_CAPTURE: RefCell<Option<ActiveCapture>> = const { RefCell::new(None) };
    static CAPTURE_BUFFERS: RefCell<Option<CaptureBuffers>> = const { RefCell::new(None) };
}

#[derive(Default)]
struct CaptureBuffers {
    pcm: Vec<f32>,
    chunks: Vec<Vec<u8>>,
}

/// Stops the mic and disconnects the graph when dropped. Owns the audio
/// callback closure, so dropping it also unregisters the listener.
struct ActiveCapture {
    stream: web_sys::MediaStream,
    context: web_sys::AudioContext,
    source: web_sys::MediaStreamAudioSourceNode,
    processor: web_sys::ScriptProcessorNode,
    listener: Option<wasm_bindgen::closure::Closure<dyn FnMut(web_sys::AudioProcessingEvent)>>,
}

impl Drop for ActiveCapture {
    fn drop(&mut self) {
        self.processor.set_onaudioprocess(None);
        self.listener = None;
        let _ = self.source.disconnect();
        let _ = self.processor.disconnect();
        let _ = self.context.close();
        let tracks = self.stream.get_audio_tracks();
        for index in 0..tracks.length() {
            if let Ok(track) = tracks.get(index).dyn_into::<web_sys::MediaStreamTrack>() {
                let _ = track.set_enabled(false);
            }
        }
    }
}

#[component]
pub fn LongRecordPanel(
    disposed: Callback<(), bool>,
    on_text_extracted: Callback<String>,
    on_error: Callback<String>,
    /// Signals for the shared transcription state (parent tab owns them).
    audio_state: RwSignal<AudioState>,
    status_text: RwSignal<Option<String>>,
    error_message: RwSignal<Option<String>>,
) -> impl IntoView {
    let i18n = use_i18n();
    let recording = RwSignal::new(false);
    let secs = RwSignal::new(0_u32);

    let ctx_for_stop = TranscribeContext {
        audio_state,
        status_text,
        error_message,
        disposed: StoredValue::new(()),
        stale_run: None,
    };

    view! {
        <div class="space-y-2" data-testid="words-long-record-panel">
            <Text size=TextSize::Small variant=TypographyVariant::Muted>
                {t!(i18n, words.audio.long_record_hint)}
            </Text>
            {move || {
                if recording.get() {
                    let stop_i18n = i18n;
                    let stop_ctx = ctx_for_stop.clone();
                    let stop_disposed = disposed.clone();
                    view! {
                        <div class="flex items-center gap-3">
                            <span class="spinner spinner-sm"></span>
                            <Text size=TextSize::Default variant=TypographyVariant::Muted>
                                {move || {
                                    i18n.get_keys()
                                        .words()
                                        .audio()
                                        .recording_secs()
                                        .inner()
                                        .to_string()
                                        .replacen("{}", &secs.get().to_string(), 1)
                                }}
                            </Text>
                            <Button
                                variant=ButtonVariant::Olive
                                on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                                    recording.set(false);
                                    let chunks = stop_recording();
                                    transcribe_recorded(
                                        stop_i18n,
                                        chunks,
                                        stop_ctx.clone(),
                                        stop_disposed.clone(),
                                        on_text_extracted,
                                        on_error,
                                    );
                                })
                                test_id="words-long-record-stop-btn"
                            >
                                {t!(i18n, words.audio.record_stop)}
                            </Button>
                        </div>
                    }.into_any()
                } else {
                    let start_i18n = i18n;
                    let start_disposed = disposed.clone();
                    view! {
                        <Button
                            variant=ButtonVariant::Ghost
                            on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                                start_recording(
                                    start_i18n,
                                    recording,
                                    error_message,
                                    secs,
                                    start_disposed.clone(),
                                );
                            })
                            test_id="words-long-record-start-btn"
                        >
                            {t!(i18n, words.audio.record_start)}
                        </Button>
                    }.into_any()
                }
            }}
        </div>
    }
}

fn start_recording(
    _i18n: leptos_i18n::I18nContext<crate::i18n::Locale>,
    recording: RwSignal<bool>,
    error_message: RwSignal<Option<String>>,
    secs: RwSignal<u32>,
    disposed: Callback<(), bool>,
) {
    error_message.set(None);
    secs.set(0);

    spawn_local(async move {
        match open_capture().await {
            Ok(active) => {
                ACTIVE_CAPTURE.with(|slot| *slot.borrow_mut() = Some(active));
                CAPTURE_BUFFERS.with(|slot| *slot.borrow_mut() = Some(CaptureBuffers::default()));
                recording.set(true);
                tick_seconds(secs, disposed);
            },
            Err(e) => {
                warn!(error = %e, "Long recording start failed");
                error_message.set(Some(e));
            },
        }
    });
}

/// Stops the capture and returns the recorded WAV chunks.
fn stop_recording() -> Vec<Vec<u8>> {
    ACTIVE_CAPTURE.with(|slot| *slot.borrow_mut() = None); // Drop = mic off
    CAPTURE_BUFFERS.with(|slot| {
        let Some(buffers) = &mut *slot.borrow_mut() else {
            return Vec::new();
        };
        let tail = std::mem::take(&mut buffers.pcm);
        let mut chunks = std::mem::take(&mut buffers.chunks);
        if !tail.is_empty() {
            chunks.push(encode_wav_s16_mono(&tail, TARGET_RATE));
        }
        chunks
    })
}

async fn open_capture() -> Result<ActiveCapture, String> {
    let window = web_sys::window().ok_or("No window")?;
    let media_devices = window
        .navigator()
        .media_devices()
        .expect("media_devices must exist");
    let mut constraints = web_sys::MediaStreamConstraints::new();
    constraints.audio(&JsValue::from_bool(true));
    let promise = media_devices
        .get_user_media_with_constraints(&constraints)
        .map_err(|e| format!("Microphone access failed: {e:?}"))?;
    let stream: web_sys::MediaStream = JsFuture::from(promise)
        .await
        .map_err(|e| format!("Microphone access denied: {e:?}"))?
        .into();

    let context = web_sys::AudioContext::new_with_context_options(
        &web_sys::AudioContextOptions::new().sample_rate(TARGET_RATE as f32),
    )
    .map_err(|e| format!("AudioContext failed: {e:?}"))?;
    let source = context
        .create_media_stream_source(&stream)
        .map_err(|e| format!("Audio source failed: {e:?}"))?;
    let processor = context
        .create_script_processor_with_buffer_size_and_number_of_input_channels_and_number_of_output_channels(
            4096,
            1,
            1,
        )
        .map_err(|e| format!("Audio processor failed: {e:?}"))?;

    // PCM accumulation + chunk pre-encoding for the whole capture lifetime.
    let listener = wasm_bindgen::closure::Closure::wrap(Box::new(
        move |event: web_sys::AudioProcessingEvent| {
            let input = event
                .input_buffer()
                .expect("audio processing event carries a buffer");
            let channels = input.number_of_channels();
            let frame = input
                .get_channel_data(0)
                .expect("channel 0 must be present")
                .to_vec();
            CAPTURE_BUFFERS.with(|slot| {
                let Some(buffers) = &mut *slot.borrow_mut() else {
                    return;
                };
                buffers.pcm.extend(mix_event_channels(&frame, channels));
                let chunk_samples = (CHUNK_SECONDS * f64::from(TARGET_RATE)).ceil() as usize;
                while buffers.pcm.len() >= chunk_samples {
                    let chunk: Vec<f32> = buffers.pcm.drain(..chunk_samples).collect();
                    buffers
                        .chunks
                        .push(encode_wav_s16_mono(&chunk, TARGET_RATE));
                }
            });
        },
    )
        as Box<dyn FnMut(web_sys::AudioProcessingEvent)>);
    processor.set_onaudioprocess(Some(listener.as_ref().unchecked_ref()));

    let source_node = source;
    Ok(ActiveCapture {
        stream,
        context,
        source: source_node,
        processor,
        listener: Some(listener),
    })
}

fn mix_event_channels(interleaved: &[f32], channels: u32) -> Vec<f32> {
    match channels {
        0 | 1 => interleaved.to_vec(),
        n => interleaved
            .chunks_exact(n as usize)
            .map(|frame| frame.iter().sum::<f32>() / n as f32)
            .collect(),
    }
}

/// Seconds ticker for the recording status line.
fn tick_seconds(secs: RwSignal<u32>, disposed: Callback<(), bool>) {
    spawn_local(async move {
        loop {
            gloo_timers::future::TimeoutFuture::new(1_000).await;
            if disposed.run(()) {
                return;
            }
            secs.update(|v| *v += 1);
        }
    });
}

/// Transcribes the recorded chunks and delivers the joined text to the
/// transcript screen.
fn transcribe_recorded(
    i18n: leptos_i18n::I18nContext<crate::i18n::Locale>,
    chunks: Vec<Vec<u8>>,
    ctx: TranscribeContext,
    disposed: Callback<(), bool>,
    on_text_extracted: Callback<String>,
    on_error: Callback<String>,
) {
    let total = chunks.len();
    ctx.audio_state.set(AudioState::Processing);
    spawn_local(async move {
        let mut texts: Vec<String> = Vec::new();
        for (index, chunk) in chunks.into_iter().enumerate() {
            if disposed.run(()) || ctx.is_stale() {
                return;
            }
            let fragment = index as u32 + 1;
            ctx.status_text.set(Some(
                i18n.get_keys()
                    .words()
                    .audio()
                    .fragment_progress()
                    .inner()
                    .to_string()
                    .replacen("{}", &fragment.to_string(), 1)
                    .replacen("{}", &total.to_string(), 1),
            ));
            match transcribe_wav_bytes(&chunk, &ctx).await {
                Ok(text) if !text.trim().is_empty() => texts.push(text),
                Ok(_) => {}, // silent chunk — skip
                Err(reason) => {
                    if disposed.run(()) {
                        return;
                    }
                    let message = i18n
                        .get_keys()
                        .words()
                        .audio()
                        .interrupted_on_fragment()
                        .inner()
                        .to_string()
                        .replacen("{}", &fragment.to_string(), 1)
                        .replacen("{}", &reason, 1);
                    ctx.error_message.set(Some(message.clone()));
                    on_error.run(message);
                    return;
                },
            }
        }
        if disposed.run(()) {
            return;
        }
        if texts.is_empty() {
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
        let joined = join_chunk_texts(&texts);
        ctx.audio_state.set(AudioState::Ready);
        ctx.status_text.set(None);
        on_text_extracted.run(joined);
    });
}

#[cfg(not(target_arch = "wasm32"))]
async fn open_capture() -> Result<ActiveCapture, String> {
    Err("Microphone capture requires WASM runtime".to_string())
}
