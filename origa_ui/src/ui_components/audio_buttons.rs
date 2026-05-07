use leptos::prelude::*;
use leptos_icons::Icon;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::Closure;

use crate::ui_components::{get_reading_from_text, is_speech_supported, speak_word_with_callback};

#[component]
pub fn AudioButtons(
    #[prop(into)] text: String,
    #[prop(optional, into)] class: Signal<String>,
    #[prop(optional, into)] test_id: Signal<String>,
    #[prop(into)] audio_src: Option<String>,
) -> impl IntoView {
    let has_content = audio_src.is_some() || !get_reading_from_text(&text).is_empty();
    let has_audio_src = audio_src.is_some();
    let is_playing = RwSignal::new(false);
    let src = audio_src;

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
                    on:click={
                        let text = text.clone();
                        let src = src.clone();
                        move |_| {
                            if is_playing.get() { return; }
                            is_playing.set(true);
                            if let Some(audio_url) = &src {
                                let Ok(audio) = web_sys::HtmlAudioElement::new_with_src(audio_url) else {
                                    is_playing.set(false);
                                    return;
                                };
                                let on_end = Closure::<dyn Fn()>::new(move || {
                                    is_playing.set(false);
                                });
                                audio.set_onended(Some(on_end.as_ref().unchecked_ref()));
                                on_end.forget();
                                let _ = audio.play();
                            } else if is_speech_supported() {
                                speak_word_with_callback(&text, 1.0, move || {
                                    is_playing.set(false);
                                });
                            }
                        }
                    }
                    disabled=move || is_playing.get() || (!has_audio_src && !is_speech_supported())
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
