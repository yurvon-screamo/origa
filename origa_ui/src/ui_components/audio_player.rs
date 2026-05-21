use leptos::prelude::*;
use leptos_icons::Icon;
use wasm_bindgen::JsCast;

use super::word_audio::{register_audio, stop_current_audio};

#[derive(Clone, Copy, PartialEq)]
enum PlaybackState {
    Idle,
    Loading,
    Playing,
}

#[component]
pub fn AudioPlayer(
    #[prop(into)] src: String,
    #[prop(optional)] autoplay: bool,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let audio_ref = NodeRef::<leptos::html::Audio>::new();
    let state = RwSignal::new(if autoplay {
        PlaybackState::Loading
    } else {
        PlaybackState::Idle
    });

    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    Effect::new(move |_| {
        if autoplay {
            if let Some(audio) = audio_ref.get() {
                stop_current_audio();
                let state_clone = state;
                register_audio(
                    audio.clone().unchecked_into(),
                    Some(Box::new(move || {
                        state_clone.set(PlaybackState::Idle);
                    })),
                    vec![],
                );
                let _ = audio.play();
                state.set(PlaybackState::Loading);
            }
        }
    });

    let toggle_play = move |_| {
        if let Some(audio) = audio_ref.get() {
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
                    let _ = audio.play();
                },
            }
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
                src=src.clone()
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
