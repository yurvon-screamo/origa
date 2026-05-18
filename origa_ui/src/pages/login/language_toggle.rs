use crate::i18n::{Locale, td_string, use_i18n};
use leptos::prelude::*;

#[component]
pub fn LoginLanguageToggle(
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let i18n = use_i18n();

    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    let en_class = Signal::derive(move || {
        if i18n.get_locale() == Locale::en {
            "text-[var(--fg-black)] border-b border-[var(--fg-black)] cursor-default"
        } else {
            "text-[var(--fg-muted)] border-b border-transparent hover:text-[var(--fg-black)] hover:border-[var(--border-light)] cursor-pointer"
        }
    });

    let ru_class = Signal::derive(move || {
        if i18n.get_locale() == Locale::ru {
            "text-[var(--fg-black)] border-b border-[var(--fg-black)] cursor-default"
        } else {
            "text-[var(--fg-muted)] border-b border-transparent hover:text-[var(--fg-black)] hover:border-[var(--border-light)] cursor-pointer"
        }
    });

    view! {
        <div
            class="flex justify-end mb-4"
            role="group"
            aria-label=move || td_string!(i18n.get_locale(), login.language_aria_label)
            data-testid=test_id_val
        >
            <div class="inline-flex items-center gap-2 font-mono text-[11px] uppercase tracking-[0.15em]">
                <button
                    type="button"
                    data-testid="login-lang-en"
                    class=move || format!(
                        "bg-transparent p-0 transition-colors duration-150 ease-in-out anima-focus-ring {}",
                        en_class.get()
                    )
                    attr:aria-current=move || if i18n.get_locale() == Locale::en { "true" } else { "false" }
                    on:click=move |_| i18n.set_locale(Locale::en)
                >
                    "EN"
                </button>

                <span class="text-[var(--border-light)] select-none pointer-events-none">"|"</span>

                <button
                    type="button"
                    data-testid="login-lang-ru"
                    class=move || format!(
                        "bg-transparent p-0 transition-colors duration-150 ease-in-out anima-focus-ring {}",
                        ru_class.get()
                    )
                    attr:aria-current=move || if i18n.get_locale() == Locale::ru { "true" } else { "false" }
                    on:click=move |_| i18n.set_locale(Locale::ru)
                >
                    "RU"
                </button>
            </div>
        </div>
    }
}
