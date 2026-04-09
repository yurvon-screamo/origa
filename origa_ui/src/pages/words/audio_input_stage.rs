use base64::Engine;
use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;

use crate::ui_components::{Alert, AlertType, Button, ButtonVariant};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) enum AudioState {
    #[default]
    Idle,
    Processing,
    Ready,
    Error,
}

fn is_tauri() -> bool {
    web_sys::window()
        .and_then(|w| js_sys::Reflect::get(&w, &JsValue::from_str("__TAURI__")).ok())
        .is_some_and(|v| !v.is_undefined() && !v.is_null())
}

fn get_tauri_invoke_fn() -> Option<js_sys::Function> {
    let window = web_sys::window()?;
    let tauri_obj = js_sys::Reflect::get(&window, &JsValue::from_str("__TAURI__")).ok()?;
    let core_mod = js_sys::Reflect::get(&tauri_obj, &JsValue::from_str("core")).ok()?;
    let invoke_fn = js_sys::Reflect::get(&core_mod, &JsValue::from_str("invoke")).ok()?;
    invoke_fn.dyn_into::<js_sys::Function>().ok()
}

async fn invoke_tauri_command(cmd: &str, payload: &str) -> Result<String, String> {
    let invoke_fn = get_tauri_invoke_fn().ok_or_else(|| "Tauri API not available".to_string())?;

    let args = js_sys::Object::new();
    js_sys::Reflect::set(
        &args,
        &JsValue::from_str("payload"),
        &JsValue::from_str(payload),
    )
    .map_err(|e| format!("Failed to set payload: {:?}", e))?;

    let result = invoke_fn
        .call2(&JsValue::UNDEFINED, &JsValue::from_str(cmd), &args)
        .map_err(|e| format!("invoke {} failed: {:?}", cmd, e))?;

    let js_val = wasm_bindgen_futures::JsFuture::from(js_sys::Promise::from(result))
        .await
        .map_err(|e| format!("{} error: {:?}", cmd, e))?;

    js_val
        .as_string()
        .ok_or_else(|| format!("{} returned non-string value", cmd))
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
        let valid_ext = name.ends_with(".wav")
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

        let max_size_mb = 50.0;
        if file.size() / (1024.0 * 1024.0) > max_size_mb {
            error_message.set(Some(format!(
                "Audio file is too large. Maximum size is {} MB.",
                max_size_mb
            )));
            return;
        }

        if !is_tauri() {
            error_message.set(Some(
                "Audio transcription is only available in the desktop app.".to_string(),
            ));
            return;
        }

        audio_state.set(AudioState::Processing);
        status_text.set(Some(format!("Transcribing {}...", name)));

        let on_text_extracted = on_text_extracted;
        let on_error = on_error;
        let audio_state_local = audio_state;
        let status_text_local = status_text;
        let error_message_local = error_message;

        spawn_local(async move {
            let file_reader = match web_sys::FileReader::new() {
                Ok(r) => r,
                Err(_) => {
                    audio_state_local.set(AudioState::Error);
                    error_message_local.set(Some("Failed to create file reader".to_string()));
                    return;
                },
            };

            if file_reader.read_as_array_buffer(&file).is_err() {
                audio_state_local.set(AudioState::Error);
                error_message_local.set(Some("Failed to start reading file".to_string()));
                return;
            }

            let load_promise = js_sys::Promise::from(JsValue::from(file_reader));
            let array_buffer = match wasm_bindgen_futures::JsFuture::from(load_promise).await {
                Ok(val) => val,
                Err(e) => {
                    audio_state_local.set(AudioState::Error);
                    error_message_local.set(Some(format!("Failed to read file: {:?}", e)));
                    return;
                },
            };

            let uint8_array = js_sys::Uint8Array::new(&array_buffer);
            let mut bytes = vec![0u8; uint8_array.length() as usize];
            uint8_array.copy_to(&mut bytes);

            let base64_data = base64::engine::general_purpose::STANDARD.encode(&bytes);

            let payload = serde_json::json!({
                "audioBase64": base64_data,
                "fileName": name
            })
            .to_string();

            let result = invoke_tauri_command("transcribe_audio", &payload).await;
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
                    AudioState::Processing => view! {
                        <div class="space-y-4">
                            <div class="text-lg font-semibold text-[var(--fg-black)] flex items-center gap-2">
                                <span class="spinner spinner-sm"></span>
                                {move || status_text.get().unwrap_or_else(|| "Processing audio...".to_string())}
                            </div>
                            <Button
                                variant=Signal::derive(|| ButtonVariant::Ghost)
                                on_click=Callback::new(move |_| audio_state.set(AudioState::Idle))
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
                                        <p class="text-xs text-[var(--fg-muted)]">WAV, MP3, WebM, M4A, OGG (max 50 MB)</p>
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
