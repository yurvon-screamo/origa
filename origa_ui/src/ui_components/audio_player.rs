use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_icons::Icon;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;

use super::word_audio::{register_audio, stop_current_audio};
use crate::repository::cdn_provider::{prefetch_blob_url, resolve_audio_url};

#[derive(Clone, Copy, PartialEq)]
enum PlaybackState {
    Idle,
    Loading,
    Playing,
}

#[component]
pub fn AudioPlayer(
    /// CDN path (e.g. `phrases/audio/ABC.opus`). Always prefetched into a
    /// `blob:` URL before being handed to `<audio>` — see
    /// `cdn_provider::resolve_audio_url` for the gzip-on-CDN root cause.
    #[prop(into)]
    path: String,
    #[prop(optional)] autoplay: bool,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let audio_ref = NodeRef::<leptos::html::Audio>::new();
    let blob_url: RwSignal<Option<String>> = RwSignal::new(None);
    let state = RwSignal::new(if autoplay {
        PlaybackState::Loading
    } else {
        PlaybackState::Idle
    });

    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    // Bug A fix: never feed the raw CDN URL to <audio> (Hikari gzip). The sync
    // fast-path returns the cached blob URL instantly when the phrase has
    // already been materialised (e.g. replay); on cache miss we await
    // `prefetch_blob_url` so `blob_url` reactively updates for autoplay.
    // `resolve_audio_url` is a pure lookup, so there is no double-fetch race.
    let path_for_prefetch = path.clone();
    if let Some(url) = resolve_audio_url(&path_for_prefetch) {
        blob_url.set(Some(url));
    } else {
        spawn_local(async move {
            match prefetch_blob_url(&path_for_prefetch).await {
                Ok(url) => blob_url.set(Some(url)),
                Err(e) => {
                    tracing::warn!(path = %path_for_prefetch, error = ?e, "AudioPlayer prefetch failed")
                },
            }
        });
    }

    // Autoplay: trigger play() once the blob URL lands and the audio element
    // is mounted. Re-runs reactively whenever `blob_url` becomes Some.
    Effect::new(move |_| {
        let Some(url) = blob_url.get() else { return };
        let Some(audio) = audio_ref.get() else { return };
        audio.set_src(&url);
        if autoplay {
            stop_current_audio();
            let state_clone = state;
            register_audio(
                audio.clone().unchecked_into(),
                Some(Box::new(move || {
                    state_clone.set(PlaybackState::Idle);
                })),
                vec![],
            );
            state.set(PlaybackState::Loading);
            let reset = move || state.set(PlaybackState::Idle);
            spawn_play_with_catch(audio, reset);
        }
    });

    let toggle_play = move |_| {
        let Some(audio) = audio_ref.get() else { return };
        match state.get() {
            PlaybackState::Playing => {
                let _ = audio.pause();
                state.set(PlaybackState::Idle);
            },
            _ => {
                stop_current_audio();
                audio.set_current_time(0.0);
                let state_clone = state;
                let audio_ref_clone = audio_ref;
                register_audio(
                    audio.clone().unchecked_into(),
                    Some(Box::new(move || {
                        if let Some(a) = audio_ref_clone.get() {
                            let _ = a.pause();
                        }
                        state_clone.set(PlaybackState::Idle);
                    })),
                    vec![],
                );
                state.set(PlaybackState::Loading);
                let reset = move || state.set(PlaybackState::Idle);
                spawn_play_with_catch(audio, reset);
            },
        }
    };

    let on_playing = move |_| {
        state.set(PlaybackState::Playing);
    };

    let on_waiting = move |_| {
        state.set(PlaybackState::Loading);
    };

    let on_pause = move |_| {
        state.set(PlaybackState::Idle);
    };

    let on_ended = move |_| {
        state.set(PlaybackState::Idle);
    };

    let on_error = move |_| {
        state.set(PlaybackState::Idle);
    };

    view! {
        <div class="audio-player flex items-center justify-center" data-testid=test_id_val>
            <audio
                node_ref=audio_ref
                preload="none"
                on:playing=on_playing
                on:waiting=on_waiting
                on:pause=on_pause
                on:ended=on_ended
                on:error=on_error
            />
            <button
                class="audio-player-btn p-3 sm:p-4 rounded-full border transition-all cursor-pointer hover:bg-[var(--bg-hover)]"
                on:click=toggle_play
            >
                {move || match state.get() {
                    PlaybackState::Idle => view! {
                        <Icon icon=icondata::LuPlay width="1.5em" height="1.5em" />
                    }
                    .into_any(),
                    PlaybackState::Loading => view! {
                        <span class="animate-spin inline-flex">
                            <Icon icon=icondata::LuLoader width="1.5em" height="1.5em" />
                        </span>
                    }
                    .into_any(),
                    PlaybackState::Playing => view! {
                        <Icon icon=icondata::LuPause width="1.5em" height="1.5em" />
                    }
                    .into_any(),
                }}
            </button>
        </div>
    }
}

/// Invoke `HTMLAudioElement.play()` and absorb Promise rejection.
///
/// `play()` returns a Promise that rejects when the element cannot decode the
/// source or when autoplay policy blocks playback (NotAllowedError). Awaiting
/// the Promise via `JsFuture` marks the rejection as handled, which keeps the
/// console clean (Bug B). Crucially, on rejection `on_reject` is invoked so the
/// caller can reset its UI state — `HTMLMediaElement.onerror` does NOT fire for
/// autoplay-policy rejections, so without this the caller would stay stuck in
/// its "loading/playing" state forever.
fn spawn_play_with_catch(audio: web_sys::HtmlAudioElement, on_reject: impl FnOnce() + 'static) {
    let promise = match audio.play() {
        Ok(p) => p,
        Err(_) => {
            on_reject();
            return;
        },
    };
    spawn_local(async move {
        if JsFuture::from(promise).await.is_err() {
            on_reject();
        }
    });
}
