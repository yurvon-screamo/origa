use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_icons::Icon;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use wasm_bindgen_futures::JsFuture;

use super::word_audio::{register_audio, stop_current_audio};
use crate::repository::cdn_provider::prefetch_blob_url;
use crate::ui_components::{get_reading_from_text, is_speech_supported, speak_word_with_callback};

#[component]
pub fn AudioButtons(
    #[prop(into)] text: String,
    #[prop(optional, into)] class: Signal<String>,
    #[prop(optional, into)] test_id: Signal<String>,
    /// CDN path (e.g. `phrases/audio/ABC.opus`). Always prefetched into a
    /// `blob:` URL before being handed to `<audio>` — see
    /// `cdn_provider::resolve_audio_url` for the gzip-on-CDN root cause.
    #[prop(into)]
    audio_path: Option<String>,
) -> impl IntoView {
    let has_content = audio_path.is_some() || !get_reading_from_text(&text).is_empty();
    let has_audio_path = audio_path.is_some();
    let is_playing = RwSignal::new(false);
    // Store inputs in RwSignals so the click closure can satisfy Leptos 0.8's
    // `TypedChildrenFn` bound (`Fn + Send + Sync`). `Rc` is neither Send nor
    // Sync; signals are Copy and yield owned values via `.get()`.
    let path: RwSignal<Option<String>> = RwSignal::new(audio_path);
    let text_signal: RwSignal<String> = RwSignal::new(text);

    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    view! {
        <Show when=move || has_content>
            <div class=move || format!("audio-buttons {}", class.get())>
                <button
                    class="audio-btn"
                    data-testid=test_id_val
                    on:click=move |_| {
                        if is_playing.get() { return; }
                        if let Some(audio_path) = path.get() {
                            // Stop synchronously before the async prefetch so the
                            // currently-playing phrase does not overlap the new
                            // one during the network round-trip.
                            stop_current_audio();
                            is_playing.set(true);
                            spawn_local(async move {
                                play_phrase_audio(&audio_path, is_playing).await;
                            });
                        } else if is_speech_supported() {
                            stop_current_audio();
                            is_playing.set(true);
                            let text_owned = text_signal.get();
                            speak_word_with_callback(&text_owned, 1.0, move || {
                                is_playing.set(false);
                            });
                        }
                    }
                    disabled=move || is_playing.get() || (!has_audio_path && !is_speech_supported())
                >
                    <Show when=move || is_playing.get() fallback=|| view! {
                        <Icon icon=icondata::LuPlay width="1em" height="1em" />
                    }>
                        <Icon icon=icondata::LuPause width="1em" height="1em" />
                    </Show>
                </button>
            </div>
        </Show>
    }
}

/// Prefetch the CDN audio into a `blob:` URL and play it through an
/// `HTMLAudioElement`. Updates `is_playing` based on lifecycle events.
///
/// Uses the same promise-absorbing pattern as `AudioPlayer` to avoid Bug B
/// (`Uncaught (in promise) NotSupportedError`). If the prefetch fails the
/// signal is reset so the user can retry.
async fn play_phrase_audio(path: &str, is_playing: RwSignal<bool>) {
    let blob_url = match prefetch_blob_url(path).await {
        Ok(url) => url,
        Err(e) => {
            tracing::warn!(path = %path, error = ?e, "AudioButtons prefetch failed");
            is_playing.set(false);
            return;
        },
    };

    let Ok(audio) = web_sys::HtmlAudioElement::new_with_src(&blob_url) else {
        is_playing.set(false);
        return;
    };
    let _ = audio.set_attribute("preload", "auto");

    stop_current_audio();

    let is_playing_end = is_playing;
    let on_end = Closure::<dyn FnMut()>::new(move || {
        is_playing_end.set(false);
    });
    audio.set_onended(Some(on_end.as_ref().unchecked_ref()));

    // Mid-stream errors (e.g. the source becomes invalid after a successful
    // start) fire neither onended nor a play() Promise rejection — the promise
    // already resolved at start. Without onerror, is_playing would stay `true`
    // and the button would be permanently stuck in "Stop". The redundant
    // set(false) relative to on_end is idempotent.
    let is_playing_error = is_playing;
    let path_for_error = path.to_string();
    let on_error = Closure::<dyn FnMut()>::new(move || {
        tracing::warn!(path = %path_for_error, "AudioButtons playback error, releasing button state");
        is_playing_error.set(false);
    });
    audio.set_onerror(Some(on_error.as_ref().unchecked_ref()));

    let is_playing_stop = is_playing;
    register_audio(
        audio.clone(),
        Some(Box::new(move || {
            is_playing_stop.set(false);
        })),
        vec![on_end, on_error],
    );

    // Bug B fix: consume the play() Promise so a rejection (decode failure,
    // autoplay policy) does not surface as an uncaught rejection. onerror does
    // NOT fire for autoplay-policy NotAllowedError, so we must reset is_playing
    // here to avoid a permanently-disabled button.
    match audio.play() {
        Ok(promise) => {
            if JsFuture::from(promise).await.is_err() {
                is_playing.set(false);
            }
        },
        Err(_) => {
            is_playing.set(false);
        },
    }
}
