use std::cell::RefCell;

use leptos::wasm_bindgen::JsCast;
use leptos::wasm_bindgen::closure::Closure;
use origa::dictionary::pitch_audio::get_audio_for_reading;
use tracing::warn;

use super::{get_reading_from_text, speak_tts_text, speak_tts_text_with_callback, stop_speech};

struct ActiveAudio {
    element: web_sys::HtmlAudioElement,
    on_stop: Option<Box<dyn Fn()>>,
    _closures: Vec<Closure<dyn FnMut()>>,
}

thread_local! {
    static CURRENT_AUDIO: RefCell<Option<ActiveAudio>> = const { RefCell::new(None) };
}

pub fn stop_current_audio() {
    let prev = CURRENT_AUDIO.with(|cell| cell.borrow_mut().take());
    if let Some(active) = prev {
        if let Some(on_stop) = active.on_stop {
            on_stop();
        }
        active.element.set_onended(None);
        active.element.set_onerror(None);
        let _ = active.element.pause();
    }
    let _ = stop_speech();
}

pub fn register_audio(
    element: web_sys::HtmlAudioElement,
    on_stop: Option<Box<dyn Fn()>>,
    closures: Vec<Closure<dyn FnMut()>>,
) {
    CURRENT_AUDIO.with(|cell| {
        *cell.borrow_mut() = Some(ActiveAudio {
            element,
            on_stop,
            _closures: closures,
        });
    });
}

fn kata_to_hira(text: &str) -> String {
    text.chars()
        .map(|c| {
            // Standard katakana range (Katakana small aio -> Ka). Does not cover
            // extended katakana (ヷヸヹヺ) which never appear in dictionary readings.
            if ('\u{30A1}'..='\u{30F6}').contains(&c) {
                char::from_u32(c as u32 - 0x60).unwrap_or(c)
            } else {
                c
            }
        })
        .collect()
}

fn create_and_play_audio(word: &str) -> Option<web_sys::HtmlAudioElement> {
    let reading = get_reading_from_text(word);
    let reading_hira = kata_to_hira(&reading);
    let entry = get_audio_for_reading(word, &reading_hira)?;
    let path = format!("/{}", entry.cdn_path());
    let url = crate::repository::cdn_provider::resolve_audio_url(&path);

    let audio = web_sys::HtmlAudioElement::new().ok()?;
    audio.set_src(&url);
    let _ = audio.set_attribute("preload", "auto");

    stop_current_audio();

    register_audio(audio.clone(), None, vec![]);

    let _ = audio.play();
    Some(audio)
}

pub fn speak_word(word: &str, rate: f32) {
    if word.is_empty() {
        return;
    }

    let audio = match create_and_play_audio(word) {
        Some(a) => a,
        None => {
            let reading = get_reading_from_text(word);
            let _ = speak_tts_text(&reading, rate);
            return;
        },
    };

    let word_owned = word.to_string();
    let on_error = Closure::<dyn FnMut()>::new(move || {
        warn!(word = %word_owned, "CDN audio failed, falling back to TTS");
        let reading = get_reading_from_text(&word_owned);
        let _ = speak_tts_text(&reading, rate);
    });
    audio.set_onerror(Some(on_error.as_ref().unchecked_ref()));

    register_audio(audio, None, vec![on_error]);
}

pub fn speak_word_with_callback<F>(word: &str, rate: f32, on_end: F)
where
    F: FnMut() + 'static,
{
    if word.is_empty() {
        return;
    }

    let audio = match create_and_play_audio(word) {
        Some(a) => a,
        None => {
            let reading = get_reading_from_text(word);
            let _ = speak_tts_text_with_callback(&reading, rate, on_end);
            return;
        },
    };

    let callback = std::rc::Rc::new(RefCell::new(Some(on_end)));

    let cb_ended = callback.clone();
    let on_ended = Closure::<dyn FnMut()>::new(move || {
        if let Some(mut cb) = cb_ended.borrow_mut().take() {
            cb();
        }
    });
    audio.set_onended(Some(on_ended.as_ref().unchecked_ref()));

    let cb_error = callback.clone();
    let word_owned = word.to_string();
    let on_error = Closure::<dyn FnMut()>::new(move || {
        warn!(word = %word_owned, "CDN audio failed (with callback), falling back to TTS");
        if let Some(cb) = cb_error.borrow_mut().take() {
            let reading = get_reading_from_text(&word_owned);
            let _ = speak_tts_text_with_callback(&reading, rate, cb);
        }
    });
    audio.set_onerror(Some(on_error.as_ref().unchecked_ref()));

    register_audio(audio, None, vec![on_ended, on_error]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kata_to_hira_converts_standard() {
        assert_eq!(kata_to_hira("ヤク"), "やく");
        assert_eq!(kata_to_hira("ア"), "あ");
        assert_eq!(kata_to_hira("ン"), "ん");
    }

    #[test]
    fn kata_to_hira_preserves_hiragana() {
        assert_eq!(kata_to_hira("やく"), "やく");
    }

    #[test]
    fn kata_to_hira_preserves_other() {
        assert_eq!(kata_to_hira("hello"), "hello");
        assert_eq!(kata_to_hira("123"), "123");
    }

    #[test]
    fn kata_to_hira_empty() {
        assert_eq!(kata_to_hira(""), "");
    }

    #[test]
    fn kata_to_hira_mixed() {
        assert_eq!(kata_to_hira("ヤクabc"), "やくabc");
    }

    #[test]
    fn kata_to_hira_long_vowel_mark() {
        assert_eq!(kata_to_hira("バー"), "ばー");
    }
}
