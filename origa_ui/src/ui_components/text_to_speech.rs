use std::collections::HashSet;

use leptos::wasm_bindgen::closure::Closure;
use leptos::wasm_bindgen::JsCast;
use origa::domain::{filter_japanese_text, furiganize_segments};
use web_sys::js_sys::Function;
use web_sys::{window, SpeechSynthesisUtterance, SpeechSynthesisVoice};

pub fn is_speech_supported() -> bool {
    window().and_then(|w| w.speech_synthesis().ok()).is_some()
}

pub fn speak_text(text: &str, rate: f32) -> Result<(), String> {
    if text.is_empty() {
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
        return "".to_string();
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
    let window = window().ok_or("Window not available")?;
    let synthesis = window
        .speech_synthesis()
        .map_err(|e| format!("Speech synthesis not supported: {:?}", e))?;
    synthesis.cancel();
    Ok(())
}
