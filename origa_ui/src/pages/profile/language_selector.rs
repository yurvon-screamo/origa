use crate::ui_components::{Button, ButtonVariant};
use leptos::prelude::*;
use origa::domain::NativeLanguage;

const LANGUAGES: &[NativeLanguage] = &[NativeLanguage::English, NativeLanguage::Russian];

#[component]
pub fn LanguageSelector(selected_language: RwSignal<NativeLanguage>) -> impl IntoView {
    view! {
        <div class="flex space-x-2 mt-2">
            <For
                each=move || LANGUAGES.to_vec()
                key=|lang| format!("{:?}", lang)
                children=move |lang| {
                    let is_selected = move || selected_language.get() == lang;
                    view! {
                        <Button
                            variant=move || if is_selected() { ButtonVariant::Olive } else { ButtonVariant::Default }
                            on_click={Callback::new(move |_| selected_language.set(lang))}
                        >
                            {format!("{:?}", lang)}
                        </Button>
                    }
                }
            />
        </div>
    }
}
