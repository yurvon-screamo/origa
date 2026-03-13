use leptos::prelude::*;
use leptos_icons::Icon;

use crate::ui_components::{get_reading_from_text, is_speech_supported, speak_text_with_callback};

#[component]
pub fn AudioButtons(
    #[prop(into)] text: String,
    #[prop(optional, into)] class: Signal<String>,
) -> impl IntoView {
    let reading = get_reading_from_text(&text);
    let has_reading = !reading.is_empty();
    let is_playing = RwSignal::new(false);

    view! {
        <Show when=move || has_reading>
            <div class=move || format!("flex gap-2 {}", class.get())>
                <button
                    class="p-1.5 rounded-full bg-blue-50 hover:bg-blue-100 text-blue-600 hover:text-blue-700 transition-colors disabled:opacity-50"
                    on:click={
                        let reading = reading.clone();
                        move |_| {
                            if is_speech_supported() && !is_playing.get() {
                                is_playing.set(true);
                                let _ = speak_text_with_callback(&reading, 1.0, move || {
                                    is_playing.set(false);
                                });
                            }
                        }
                    }
                    disabled=move || is_playing.get() || !is_speech_supported()
                >
                    <Show when=move || is_playing.get() fallback=|| view! {
                        <Icon icon=icondata::LuVolume width="1.25em" height="1.25em" />
                    }>
                        <span class="inline-block animate-spin">
                            <Icon icon=icondata::LuLoader width="1.25em" height="1.25em" />
                        </span>
                    </Show>
                </button>
            </div>
        </Show>
    }
}
