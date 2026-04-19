use leptos::prelude::*;
use leptos_icons::Icon;

#[component]
pub fn AudioPlayer(
    #[prop(into)] src: String,
    #[prop(optional)] autoplay: bool,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let audio_ref = NodeRef::<leptos::html::Audio>::new();
    let is_playing = RwSignal::new(false);
    let has_loaded = RwSignal::new(false);

    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    Effect::new(move |_| {
        if autoplay {
            if let Some(audio) = audio_ref.get() {
                let _ = audio.play();
                is_playing.set(true);
                has_loaded.set(true);
            }
        }
    });

    let toggle_play = move |_| {
        if let Some(audio) = audio_ref.get() {
            if is_playing.get() {
                let _ = audio.pause();
                is_playing.set(false);
            } else {
                has_loaded.set(true);
                let _ = audio.play();
                is_playing.set(true);
            }
        }
    };

    let on_ended = move |_| {
        is_playing.set(false);
    };

    view! {
        <div class="audio-player flex items-center justify-center" data-testid=test_id_val>
            <audio
                node_ref=audio_ref
                src=move || if has_loaded.get() || autoplay { src.clone() } else { String::new() }
                preload=move || if autoplay { "auto" } else { "none" }
                on:ended=on_ended
            />
            <button
                class="audio-player-btn p-3 sm:p-4 rounded-full border transition-all cursor-pointer hover:bg-[var(--bg-hover)]"
                on:click=toggle_play
            >
                <Show when=move || is_playing.get() fallback=|| view! {
                    <Icon icon=icondata::LuPlay width="1.5em" height="1.5em" />
                }>
                    <Icon icon=icondata::LuPause width="1.5em" height="1.5em" />
                </Show>
            </button>
        </div>
    }
}
