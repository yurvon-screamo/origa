use std::cell::RefCell;

use leptos::wasm_bindgen::JsCast;
use leptos::wasm_bindgen::closure::Closure;
use origa::dictionary::pitch_audio::get_audio_for_word;
use tracing::warn;

use super::{get_reading_from_text, speak_tts_text, speak_tts_text_with_callback, stop_speech};
use crate::core::config::cdn_url;

struct ActiveAudio {
    element: web_sys::HtmlAudioElement,
    on_stop: Option<Box<dyn Fn()>>,
}

thread_local! {
    static CURRENT_AUDIO: RefCell<Option<ActiveAudio>> = const { RefCell::new(None) };
    static AUDIO_CLOSURES: RefCell<Vec<Closure<dyn FnMut()>>> = RefCell::new(Vec::new());
}

pub fn stop_current_audio() {
    CURRENT_AUDIO.with(|cell| {
        if let Some(active) = cell.borrow_mut().take() {
            if let Some(on_stop) = active.on_stop {
                on_stop();
            }
            let _ = active.element.pause();
            active.element.set_src("");
        }
    });
    AUDIO_CLOSURES.with(|cell| {
        cell.borrow_mut().clear();
    });
    let _ = stop_speech();
}

pub fn register_audio(element: web_sys::HtmlAudioElement, on_stop: Option<Box<dyn Fn()>>) {
    CURRENT_AUDIO.with(|cell| {
        *cell.borrow_mut() = Some(ActiveAudio { element, on_stop });
    });
}

pub fn store_closure(closure: Closure<dyn FnMut()>) {
    AUDIO_CLOSURES.with(|cell| {
        cell.borrow_mut().push(closure);
    });
}

fn create_and_play_audio(word: &str) -> Option<web_sys::HtmlAudioElement> {
    let entry = get_audio_for_word(word)?;
    let path = format!("/{}", entry.cdn_path());
    let url = cdn_url(&path);

    let audio = web_sys::HtmlAudioElement::new().ok()?;
    audio.set_src(&url);
    let _ = audio.set_attribute("preload", "auto");

    stop_current_audio();

    let audio_clone = audio.clone();
    CURRENT_AUDIO.with(|cell| {
        *cell.borrow_mut() = Some(ActiveAudio {
            element: audio_clone,
            on_stop: None,
        });
    });

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
    let _ = audio.add_event_listener_with_callback("error", on_error.as_ref().unchecked_ref());
    AUDIO_CLOSURES.with(|cell| {
        cell.borrow_mut().push(on_error);
    });
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
    let _ = audio.add_event_listener_with_callback("ended", on_ended.as_ref().unchecked_ref());
    AUDIO_CLOSURES.with(|cell| {
        cell.borrow_mut().push(on_ended);
    });

    let cb_error = callback.clone();
    let word_owned = word.to_string();
    let on_error = Closure::<dyn FnMut()>::new(move || {
        warn!(word = %word_owned, "CDN audio failed (with callback), falling back to TTS");
        if let Some(cb) = cb_error.borrow_mut().take() {
            let reading = get_reading_from_text(&word_owned);
            let _ = speak_tts_text_with_callback(&reading, rate, cb);
        }
    });
    let _ = audio.add_event_listener_with_callback("error", on_error.as_ref().unchecked_ref());
    AUDIO_CLOSURES.with(|cell| {
        cell.borrow_mut().push(on_error);
    });
}
