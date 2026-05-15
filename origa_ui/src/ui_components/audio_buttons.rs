use leptos::prelude::*;
use leptos_icons::Icon;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::Closure;

use super::word_audio::{register_audio, stop_current_audio, store_closure};
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

    on_cleanup(move || {
        stop_current_audio();
    });

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
                                stop_current_audio();
                                let Ok(audio) = web_sys::HtmlAudioElement::new_with_src(audio_url) else {
                                    is_playing.set(false);
                                    return;
                                };
                                let is_playing_clone = is_playing;
                                let on_end = Closure::<dyn FnMut()>::new(move || {
                                    is_playing_clone.set(false);
                                });
                                audio.set_onended(Some(on_end.as_ref().unchecked_ref()));
                                store_closure(on_end);

                                let is_playing_clone2 = is_playing;
                                register_audio(audio.clone(), Some(Box::new(move || {
                                    is_playing_clone2.set(false);
                                })));

                                let _ = audio.play();
                            } else if is_speech_supported() {
                                stop_current_audio();
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
