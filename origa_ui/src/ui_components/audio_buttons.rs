use leptos::prelude::*;
use leptos_icons::Icon;

use crate::ui_components::{get_reading_from_text, is_speech_supported, speak_text};

#[component]
pub fn AudioButtons(
    #[prop(into)] text: String,
    #[prop(optional, into)] class: Signal<String>,
) -> impl IntoView {
    let reading = get_reading_from_text(&text);
    let has_reading = !reading.is_empty();

    view! {
        <Show when=move || has_reading>
            <div class=move || format!("flex gap-2 {}", class.get())>
                <button
                    class="btn btn-ghost px-2 py-1"
                    on:click={
                        let reading = reading.clone();
                        move |_| {
                            if is_speech_supported() {
                                let _ = speak_text(&reading, 1.0);
                            }
                        }
                    }
                    disabled=move || !is_speech_supported()
                >
                    <Icon icon=icondata::LuVolume2 width="1.25em" height="1.25em" />
                </button>
            </div>
        </Show>
    }
}
