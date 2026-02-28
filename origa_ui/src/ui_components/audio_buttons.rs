use leptos::prelude::*;

use crate::ui_components::{get_reading_from_text, is_speech_supported, speak_text};

#[component]
pub fn AudioButtons(
    #[prop(into)] text: String,
    #[prop(optional, into)] class: Signal<String>,
) -> impl IntoView {
    let text_normal = text.clone();
    let text_slow = text.clone();

    view! {
        <div class=move || format!("flex gap-1 {}", class.get())>
            <button
                class="btn btn-ghost btn-sm px-2 py-1 text-base"
                on:click=move |_| {
                    let reading = get_reading_from_text(&text_normal);
                    if is_speech_supported() {
                        let _ = speak_text(&reading, 1.0);
                    }
                }
                disabled=move || !is_speech_supported()
            >
                "🔊"
            </button>
            <button
                class="btn btn-ghost btn-sm px-2 py-1 text-base"
                on:click=move |_| {
                    let reading = get_reading_from_text(&text_slow);
                    if is_speech_supported() {
                        let _ = speak_text(&reading, 0.5);
                    }
                }
                disabled=move || !is_speech_supported()
            >
                "🐌"
            </button>
        </div>
    }
}
