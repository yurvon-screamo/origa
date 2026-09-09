use crate::i18n::{td_string, use_i18n};
use leptos::prelude::*;
use origa::domain::NativeLanguage;

const ACTIVE_CLASS: &str =
    "text-[var(--fg-black)] border-b border-[var(--fg-black)] cursor-default";
const INACTIVE_CLASS: &str = "text-[var(--fg-muted)] border-b border-transparent hover:text-[var(--fg-black)] hover:border-[var(--border-light)] cursor-pointer";

/// (language, test-id slug, label). Order defines the on-screen order of the
/// toggle buttons.
const LANGUAGES: [(NativeLanguage, &str, &str); 4] = [
    (NativeLanguage::English, "en", "EN"),
    (NativeLanguage::Russian, "ru", "RU"),
    (NativeLanguage::Korean, "ko", "KO"),
    (NativeLanguage::Vietnamese, "vi", "VI"),
];

#[component]
pub fn NativeLanguageToggle(
    selected_language: RwSignal<NativeLanguage>,
    #[prop(optional, into)] test_id: Signal<String>,
    #[prop(optional)] on_change: Option<Callback<NativeLanguage>>,
) -> impl IntoView {
    let i18n = use_i18n();

    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    view! {
        <div
            class="inline-flex items-center gap-2 font-mono text-[11px] uppercase tracking-[0.15em]"
            role="group"
            aria-label=move || td_string!(i18n.get_locale(), common.language_aria_label)
            data-testid=test_id_val
        >
            {LANGUAGES
                .iter()
                .enumerate()
                .map(|(index, &(lang, slug, label))| {
                    let class = Signal::derive(move || {
                        if selected_language.get() == lang { ACTIVE_CLASS } else { INACTIVE_CLASS }
                    });
                    let separator = (index > 0).then(|| {
                        view! {
                            <span class="text-[var(--border-light)] select-none pointer-events-none">"|"</span>
                        }
                    });
                    view! {
                        {separator}
                        <button
                            type="button"
                            data-testid=format!("lang-toggle-{slug}")
                            class=move || format!(
                                "bg-transparent p-0 transition-colors duration-150 ease-in-out anima-focus-ring {}",
                                class.get()
                            )
                            aria-current=move || if selected_language.get() == lang { "true" } else { "false" }
                            on:click=move |_| {
                                selected_language.set(lang);
                                if let Some(cb) = &on_change { cb.run(lang); }
                            }
                        >
                            {label}
                        </button>
                    }
                })
                .collect::<Vec<_>>()}
        </div>
    }
}
