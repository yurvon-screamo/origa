use crate::i18n::{locale_to_native_language, native_language_to_locale, use_i18n};
use crate::ui_components::NativeLanguageToggle;
use leptos::prelude::*;

#[component]
pub fn LoginLanguageToggle(#[prop(optional, into)] test_id: Signal<String>) -> impl IntoView {
    let i18n = use_i18n();
    let selected_language = RwSignal::new(locale_to_native_language(
        &i18n.get_locale_untracked(),
    ));

    Effect::new(move |_| {
        let lang = selected_language.get();
        i18n.set_locale(native_language_to_locale(&lang));
    });

    view! {
        <div class="flex justify-end mb-4">
            <NativeLanguageToggle selected_language=selected_language test_id=test_id />
        </div>
    }
}
