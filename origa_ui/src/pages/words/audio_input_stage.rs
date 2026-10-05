//! Audio-tab UI of the add-words drawer.
//!
//! File picking and progress rendering only — validation and transcription
//! live in [`super::audio_transcribe`], shared with the inbox intake path.

use leptos::prelude::*;
use leptos::task::spawn_local;

use super::audio_transcribe::{
    AudioState, TranscribeContext, cancel_whisper_loading, file_from_change_event, transcribe_file,
};
use crate::i18n::use_i18n;
use crate::ui_components::{Alert, AlertType, Button, ButtonVariant};

use super::audio_live_recorder::AudioLiveRecorder;

#[component]
pub(super) fn AudioInputStage(
    is_open: Signal<bool>,
    on_text_extracted: Callback<String>,
    on_error: Callback<String>,
    on_switch_to_text: Callback<()>,
) -> impl IntoView {
    let i18n = use_i18n();
    let audio_state = RwSignal::new(AudioState::Idle);
    let error_message = RwSignal::new(None::<String>);
    let status_text = RwSignal::new(None::<String>);
    // Guideline 4.2.3(ii): disclose the Whisper fallback download size before
    // the user commits to transcription. The native device-ai path (macOS,
    // iOS, Android) downloads nothing, so the notice is only shown when the
    // capabilities query reports native ASR unavailable.
    let needs_model_download = RwSignal::new(false);
    let disposed = StoredValue::new(());
    spawn_local(async move {
        let native_available =
            crate::core::device_ai::available(crate::core::device_ai::Feature::SpeechRecognition)
                .await;
        needs_model_download.set(!native_available);
    });

    Effect::new(move |_| {
        if !is_open.get() {
            audio_state.set(AudioState::Idle);
            error_message.set(None);
            status_text.set(None);
        }
    });

    let handle_file = move |file: web_sys::File| {
        transcribe_file(
            i18n,
            file,
            TranscribeContext {
                audio_state,
                status_text,
                error_message,
                disposed,
            },
            on_text_extracted,
            on_error,
        );
    };

    let on_change = move |ev: web_sys::Event| {
        if let Some(file) = file_from_change_event(ev) {
            handle_file(file);
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
                                {move || status_text.get().unwrap_or_else(|| i18n.get_keys().words().audio().processing().inner().to_string())}
                            </div>
                            <Button
                                variant=Signal::derive(|| ButtonVariant::Ghost)
                                on_click=Callback::new(move |_| {
                                    audio_state.set(AudioState::Idle);
                                    cancel_whisper_loading();
                                })
                            >
                                {move || i18n.get_keys().common().cancel().inner().to_string()}
                            </Button>
                        </div>
                    }.into_any(),
                    _ => view! {
                        <>
                            <div class="border-2 border-dashed p-8 text-center transition-colors cursor-pointer border-[var(--border-dark)] hover:border-[var(--accent-olive)]/50">
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
                                        <p class="text-sm text-[var(--fg-muted)]">{i18n.get_keys().words().audio().drop_zone().inner().to_string()}</p>
                                        <p class="text-xs text-[var(--fg-muted)]">{i18n.get_keys().words().audio().file_type().inner().to_string()}</p>
                                        {move || {
                                            needs_model_download
                                                .get()
                                                .then(|| {
                                                    view! {
                                                        <p class="text-xs text-[var(--fg-muted)]">
                                                            {i18n.get_keys().words().audio().model_download_notice().inner().to_string()}
                                                        </p>
                                                    }
                                                })
                                        }}
                                    </div>
                                </label>
                            </div>
                            <AudioLiveRecorder
                                on_text_extracted
                                on_error
                            />
                            {move || {
                                error_message.get().map(move |msg| view! {
                                    <div>
                                        <Alert
                                            alert_type=Signal::derive(|| AlertType::Warning)
                                            title=Signal::derive(move || i18n.get_keys().words().audio().transcription_failed().inner().to_string())
                                            message=Signal::derive(move || msg.clone())
                                        />
                                        <Button
                                            variant=ButtonVariant::Ghost
                                            on_click=Callback::new(move |_| on_switch_to_text.run(()))
                                        >
                                            {i18n.get_keys().words().audio().enter_manually().inner().to_string()}
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
