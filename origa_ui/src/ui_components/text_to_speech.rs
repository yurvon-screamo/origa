use std::cell::{Cell, RefCell};
use std::collections::HashSet;

use leptos::task::spawn_local;
use leptos::wasm_bindgen::JsCast;
use leptos::wasm_bindgen::JsValue;
use leptos::wasm_bindgen::closure::Closure;
use origa::domain::{filter_japanese_text, furiganize_segments};
use tracing::warn;
use web_sys::js_sys::Function;
use web_sys::{SpeechSynthesisUtterance, SpeechSynthesisVoice, window};

thread_local! {
    static TTS_CALLBACK: RefCell<Option<Box<dyn FnMut()>>> = const { RefCell::new(None) };
    static TTS_LISTENER_REGISTERED: Cell<bool> = const { Cell::new(false) };
}

fn is_tauri() -> bool {
    web_sys::window()
        .and_then(|w| js_sys::Reflect::get(&w, &JsValue::from_str("__TAURI__")).ok())
        .is_some_and(|v| !v.is_undefined() && !v.is_null())
}

fn get_tauri_invoke_fn() -> Result<js_sys::Function, String> {
    let window = web_sys::window().ok_or("Window not available")?;
    let tauri_obj = js_sys::Reflect::get(&window, &JsValue::from_str("__TAURI__"))
        .map_err(|_| "Tauri API not available")?;
    let core_mod = js_sys::Reflect::get(&tauri_obj, &JsValue::from_str("core"))
        .map_err(|_| "Tauri core module not available")?;
    js_sys::Reflect::get(&core_mod, &JsValue::from_str("invoke"))
        .map_err(|_| "Tauri core.invoke not available")?
        .dyn_into::<js_sys::Function>()
        .map_err(|_| "core.invoke is not a function".to_string())
}

fn ensure_tauri_listener_registered() {
    TTS_LISTENER_REGISTERED.with(|flag| {
        if flag.get() {
            return;
        }

        let Some(window) = web_sys::window() else {
            return;
        };
        let Ok(tauri_obj) = js_sys::Reflect::get(&window, &JsValue::from_str("__TAURI__")) else {
            return;
        };
        let Ok(event_mod) = js_sys::Reflect::get(&tauri_obj, &JsValue::from_str("event")) else {
            return;
        };
        let Ok(listen_fn) = js_sys::Reflect::get(&event_mod, &JsValue::from_str("listen")) else {
            return;
        };
        let Ok(listen_fn) = listen_fn.dyn_into::<js_sys::Function>() else {
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
    let invoke_fn = get_tauri_invoke_fn()?;

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
    let invoke_fn = get_tauri_invoke_fn()?;

    let result = invoke_fn
        .call1(&JsValue::UNDEFINED, &JsValue::from_str("plugin:tts|stop"))
        .map_err(|e| format!("invoke plugin:tts|stop failed: {:?}", e))?;

    wasm_bindgen_futures::JsFuture::from(js_sys::Promise::from(result))
        .await
        .map(|_| ())
        .map_err(|e| format!("plugin:tts|stop error: {:?}", e))
}

pub fn is_speech_supported() -> bool {
    if is_tauri() {
        return true;
    }
    window().and_then(|w| w.speech_synthesis().ok()).is_some()
}

pub fn speak_text(text: &str, rate: f32) -> Result<(), String> {
    if text.is_empty() {
        return Ok(());
    }

    if is_tauri() {
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

pub fn speak_text_with_callback<F>(text: &str, rate: f32, on_end: F) -> Result<(), String>
where
    F: FnMut() + 'static,
{
    if text.is_empty() {
        return Ok(());
    }

    if is_tauri() {
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

pub fn get_reading_from_text_with_known_kanji(text: &str, known_kanji: &HashSet<String>) -> String {
    let filtered_text = filter_japanese_text(text);
    if filtered_text.is_empty() {
        return String::new();
    }
    furiganize_segments(&filtered_text, known_kanji)
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
        .unwrap_or_else(|_| filtered_text)
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
    if is_tauri() {
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
