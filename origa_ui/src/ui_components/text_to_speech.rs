use std::cell::{Cell, RefCell};
use std::collections::HashSet;

use crate::core::tauri;
use leptos::task::spawn_local;
use leptos::wasm_bindgen::JsCast;
use leptos::wasm_bindgen::JsValue;
use leptos::wasm_bindgen::closure::Closure;
use origa::domain::furiganize_segments;
use tracing::warn;
use web_sys::js_sys::Function;
use web_sys::{SpeechSynthesisUtterance, SpeechSynthesisVoice, window};

thread_local! {
    static TTS_CALLBACK: RefCell<Option<Box<dyn FnMut()>>> = const { RefCell::new(None) };
    static TTS_LISTENER_REGISTERED: Cell<bool> = const { Cell::new(false) };
}

fn ensure_tauri_listener_registered() {
    TTS_LISTENER_REGISTERED.with(|flag| {
        if flag.get() {
            return;
        }

        let Some(listen_fn) = tauri::event_listen_fn() else {
            return;
        };

        let closure = Closure::<dyn FnMut(JsValue)>::new(|_| {
            TTS_CALLBACK.with(|cell| {
                if let Some(mut cb) = cell.borrow_mut().take() {
                    cb();
                }
            });
        });

        let event_name = JsValue::from_str("tts://speech:finish");
        let _ = listen_fn.call2(&JsValue::UNDEFINED, &event_name, closure.as_ref());
        closure.forget();
        flag.set(true);
    });
}

async fn invoke_tauri_speak(text: &str, rate: f32) -> Result<(), String> {
    let invoke_fn = tauri::invoke_fn().ok_or("Tauri invoke not available")?;

    let payload = js_sys::Object::new();
    js_sys::Reflect::set(
        &payload,
        &JsValue::from_str("text"),
        &JsValue::from_str(text),
    )
    .map_err(|e| format!("Failed to set text: {:?}", e))?;
    js_sys::Reflect::set(
        &payload,
        &JsValue::from_str("language"),
        &JsValue::from_str("ja-JP"),
    )
    .map_err(|e| format!("Failed to set language: {:?}", e))?;
    js_sys::Reflect::set(
        &payload,
        &JsValue::from_str("rate"),
        &JsValue::from_f64(rate as f64),
    )
    .map_err(|e| format!("Failed to set rate: {:?}", e))?;
    js_sys::Reflect::set(
        &payload,
        &JsValue::from_str("pitch"),
        &JsValue::from_f64(1.0),
    )
    .map_err(|e| format!("Failed to set pitch: {:?}", e))?;
    js_sys::Reflect::set(
        &payload,
        &JsValue::from_str("volume"),
        &JsValue::from_f64(1.0),
    )
    .map_err(|e| format!("Failed to set volume: {:?}", e))?;
    js_sys::Reflect::set(
        &payload,
        &JsValue::from_str("queueMode"),
        &JsValue::from_str("flush"),
    )
    .map_err(|e| format!("Failed to set queueMode: {:?}", e))?;

    let args = js_sys::Object::new();
    js_sys::Reflect::set(&args, &JsValue::from_str("payload"), &payload)
        .map_err(|e| format!("Failed to set payload: {:?}", e))?;

    let result = invoke_fn
        .call2(
            &JsValue::UNDEFINED,
            &JsValue::from_str("plugin:tts|speak"),
            &args,
        )
        .map_err(|e| format!("invoke plugin:tts|speak failed: {:?}", e))?;

    wasm_bindgen_futures::JsFuture::from(js_sys::Promise::from(result))
        .await
        .map(|_| ())
        .map_err(|e| format!("plugin:tts|speak error: {:?}", e))
}

async fn invoke_tauri_stop() -> Result<(), String> {
    let invoke_fn = tauri::invoke_fn().ok_or("Tauri invoke not available")?;

    let result = invoke_fn
        .call1(&JsValue::UNDEFINED, &JsValue::from_str("plugin:tts|stop"))
        .map_err(|e| format!("invoke plugin:tts|stop failed: {:?}", e))?;

    wasm_bindgen_futures::JsFuture::from(js_sys::Promise::from(result))
        .await
        .map(|_| ())
        .map_err(|e| format!("plugin:tts|stop error: {:?}", e))
}

pub fn is_speech_supported() -> bool {
    if tauri::is_tauri() {
        return true;
    }
    window().and_then(|w| w.speech_synthesis().ok()).is_some()
}

pub fn speak_tts_text(text: &str, rate: f32) -> Result<(), String> {
    if text.is_empty() {
        return Ok(());
    }

    if tauri::is_tauri() {
        let text_owned = text.to_string();
        spawn_local(async move {
            if let Err(e) = invoke_tauri_speak(&text_owned, rate).await {
                warn!("TTS speak error: {}", e);
            }
        });
        return Ok(());
    }

    let window = window().ok_or("Window not available")?;
    let synthesis = window
        .speech_synthesis()
        .map_err(|e| format!("Speech synthesis not supported: {:?}", e))?;

    synthesis.cancel();

    let utterance = SpeechSynthesisUtterance::new()
        .map_err(|e| format!("Failed to create utterance: {:?}", e))?;
    utterance.set_text(text);
    utterance.set_rate(rate);
    utterance.set_lang("ja-JP");

    if let Some(voice) = get_japanese_voice(&synthesis) {
        utterance.set_voice(Some(&voice));
    }

    synthesis.speak(&utterance);
    Ok(())
}

pub fn speak_tts_text_with_callback<F>(text: &str, rate: f32, on_end: F) -> Result<(), String>
where
    F: FnMut() + 'static,
{
    if text.is_empty() {
        return Ok(());
    }

    if tauri::is_tauri() {
        let text_owned = text.to_string();
        TTS_CALLBACK.with(|cell| {
            *cell.borrow_mut() = Some(Box::new(on_end));
        });
        ensure_tauri_listener_registered();

        spawn_local(async move {
            if let Err(e) = invoke_tauri_speak(&text_owned, rate).await {
                warn!("TTS speak error: {}", e);
            }
        });
        return Ok(());
    }

    let window = window().ok_or("Window not available")?;
    let synthesis = window
        .speech_synthesis()
        .map_err(|e| format!("Speech synthesis not supported: {:?}", e))?;

    synthesis.cancel();

    let utterance = SpeechSynthesisUtterance::new()
        .map_err(|e| format!("Failed to create utterance: {:?}", e))?;
    utterance.set_text(text);
    utterance.set_rate(rate);
    utterance.set_lang("ja-JP");

    if let Some(voice) = get_japanese_voice(&synthesis) {
        utterance.set_voice(Some(&voice));
    }

    let closure = Closure::<dyn FnMut()>::new(on_end);
    if let Some(func) = closure.as_ref().dyn_ref::<Function>() {
        utterance.set_onend(Some(func));
    }
    closure.forget();

    synthesis.speak(&utterance);
    Ok(())
}

pub fn get_reading_from_text(text: &str) -> String {
    get_reading_from_text_with_known_kanji(text, &HashSet::new())
}

pub fn get_reading_from_text_with_known_kanji(text: &str, known_kanji: &HashSet<char>) -> String {
    if text.trim().is_empty() {
        return String::new();
    }
    furiganize_segments(text, known_kanji)
        .map(|segments| {
            segments
                .iter()
                .map(|seg| {
                    seg.reading()
                        .map(|r| r.to_string())
                        .unwrap_or_else(|| seg.text().to_string())
                })
                .collect::<String>()
        })
        .unwrap_or_else(|_| text.to_string())
}

fn get_japanese_voice(synthesis: &web_sys::SpeechSynthesis) -> Option<SpeechSynthesisVoice> {
    let voices = synthesis.get_voices();
    voices.iter().find_map(|v| {
        let voice: SpeechSynthesisVoice = v.dyn_into().ok()?;
        if voice.lang().starts_with("ja") {
            Some(voice)
        } else {
            None
        }
    })
}

pub fn stop_speech() -> Result<(), String> {
    if tauri::is_tauri() {
        spawn_local(async {
            if let Err(e) = invoke_tauri_stop().await {
                warn!("TTS stop error: {}", e);
            }
        });
        return Ok(());
    }

    let window = window().ok_or("Window not available")?;
    let synthesis = window
        .speech_synthesis()
        .map_err(|e| format!("Speech synthesis not supported: {:?}", e))?;
    synthesis.cancel();
    Ok(())
}
