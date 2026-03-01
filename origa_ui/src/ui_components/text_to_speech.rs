use leptos::wasm_bindgen::JsCast;
use origa::domain::furiganize_segments;
use web_sys::{SpeechSynthesisUtterance, SpeechSynthesisVoice, window};

pub fn is_speech_supported() -> bool {
    window().and_then(|w| w.speech_synthesis().ok()).is_some()
}

pub fn speak_text(text: &str, rate: f32) -> Result<(), String> {
    let window = window().ok_or("Window not available")?;
    let synthesis = window
        .speech_synthesis()
        .map_err(|e| format!("Speech synthesis not supported: {:?}", e))?;

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

pub fn get_reading_from_text(text: &str) -> String {
    furiganize_segments(text)
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
    let window = window().ok_or("Window not available")?;
    let synthesis = window
        .speech_synthesis()
        .map_err(|e| format!("Speech synthesis not supported: {:?}", e))?;
    synthesis.cancel();
    Ok(())
}
