//! Аккордеон «Статистика» на home (fl2O037UR7): свёрнут по умолчанию,
//! заголовок-кнопка переключает видимость графиков (активность + точность).

use crate::i18n::{td_string, use_i18n};
use leptos::prelude::*;

#[component]
pub fn StatsSection(
    #[prop(optional, into)] test_id: Signal<String>,
    children: Children,
) -> impl IntoView {
    let i18n = use_i18n();
    let expanded = RwSignal::new(false);

    let section_prefix = move || {
        let val = test_id.get();
        if val.is_empty() {
            "home-stats".to_string()
        } else {
            val
        }
    };

    // Children — FnOnce: рендерим контент один раз, видимость переключаем
    // классом (графики маунтятся сразу, показываются по клику).
    let content = children();
    let hidden = move || if expanded.get() { "" } else { "hidden" };

    view! {
        <section data-testid=move || test_id.get()>
            <button
                class="flex w-full items-center justify-between cursor-pointer"
                on:click=move |_| expanded.update(|v| *v = !*v)
                data-testid=move || format!("{}-toggle", section_prefix())
            >
                <span class="text-xs text-[var(--fg-muted)] uppercase tracking-widest">
                    {move || {
                        let locale = i18n.get_locale();
                        td_string!(locale, home.stats_section_title)
                    }}
                </span>
                <span class="font-mono text-xs text-[var(--fg-muted)]">
                    {move || if expanded.get() { "▾" } else { "▸" }}
                </span>
            </button>

            <div
                class=move || format!("mt-4 space-y-6 {}", hidden())
                data-testid=move || format!("{}-content", section_prefix())
            >
                {content}
            </div>
        </section>
    }
}
