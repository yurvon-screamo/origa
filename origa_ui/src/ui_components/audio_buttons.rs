use leptos::prelude::*;
use leptos_icons::Icon;

use crate::ui_components::{get_reading_from_text, is_speech_supported, speak_word_with_callback};

#[component]
pub fn AudioButtons(
    #[prop(into)] text: String,
    #[prop(optional, into)] class: Signal<String>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let has_reading = !get_reading_from_text(&text).is_empty();
    let is_playing = RwSignal::new(false);

    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    view! {
        <Show when=move || has_reading>
            <div class=move || format!("audio-buttons {}", class.get())>
                <button
                    class="audio-btn"
                    data-testid=test_id_val
                    on:click={
                        let text = text.clone();
                        move |_| {
                            if is_speech_supported() && !is_playing.get() {
                                is_playing.set(true);
                                speak_word_with_callback(&text, 1.0, move || {
                                    is_playing.set(false);
                                });
                            }
                        }
                    }
                    disabled=move || is_playing.get() || !is_speech_supported()
                >
                    <Show when=move || is_playing.get() fallback=|| view! {
                        <Icon icon=icondata::LuVolume width="1em" height="1em" />
                    }>
                        <span class="audio-btn-spin">
                            <Icon icon=icondata::LuLoader width="1em" height="1em" />
                        </span>
                    </Show>
                </button>
            </div>
        </Show>
    }
}
