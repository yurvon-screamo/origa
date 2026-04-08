use crate::i18n::*;
use crate::ui_components::{Button, ButtonVariant};
use leptos::prelude::*;
use origa::domain::NativeLanguage;

const LANGUAGES: &[NativeLanguage] = &[NativeLanguage::English, NativeLanguage::Russian];

#[component]
pub fn LanguageSelector(selected_language: RwSignal<NativeLanguage>) -> impl IntoView {
    let i18n = use_i18n();

    view! {
        <div class="flex space-x-2 mt-2">
            <For
                each=move || LANGUAGES.to_vec()
                key=|lang| format!("{:?}", lang)
                children=move |lang| {
                    let is_selected = move || selected_language.get() == lang;
                    let lang_str = format!("{:?}", lang).to_lowercase();
                    let is_english = lang == NativeLanguage::English;
                    view! {
                        <Button
                            variant=move || if is_selected() { ButtonVariant::Olive } else { ButtonVariant::Default }
                            on_click={Callback::new(move |_| selected_language.set(lang))}
                            test_id=Signal::derive(move || format!("profile-lang-{}", lang_str))
                        >
                            <Show when={move || is_english}>
                                {t!(i18n, profile.language_english)}
                            </Show>
                            <Show when={move || !is_english}>
                                {t!(i18n, profile.language_russian)}
                            </Show>
                        </Button>
                    }
                }
            />
        </div>
    }
}
