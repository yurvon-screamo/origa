use crate::i18n::{Locale, use_i18n};
use crate::ui_components::{Text, TextSize, TypographyVariant};
use leptos::prelude::*;
use leptos_i18n::I18nContext;
use origa::domain::DailyLoad;

const DAILY_LOAD_OPTIONS: &[DailyLoad] = &[
    DailyLoad::Light,
    DailyLoad::Medium,
    DailyLoad::Hard,
    DailyLoad::Impossible,
];

fn load_label(i18n: &I18nContext<Locale>, load: &DailyLoad) -> String {
    match load {
        DailyLoad::Light => i18n.get_keys().shared().load_light().inner().to_string(),
        DailyLoad::Medium => i18n.get_keys().shared().load_medium().inner().to_string(),
        DailyLoad::Hard => i18n.get_keys().shared().load_hard().inner().to_string(),
        DailyLoad::Impossible => i18n
            .get_keys()
            .shared()
            .load_impossible()
            .inner()
            .to_string(),
    }
}

fn load_description(i18n: &I18nContext<Locale>, load: &DailyLoad) -> String {
    match load {
        DailyLoad::Light => i18n
            .get_keys()
            .shared()
            .load_light_desc()
            .inner()
            .to_string(),
        DailyLoad::Medium => i18n
            .get_keys()
            .shared()
            .load_medium_desc()
            .inner()
            .to_string(),
        DailyLoad::Hard => i18n
            .get_keys()
            .shared()
            .load_hard_desc()
            .inner()
            .to_string(),
        DailyLoad::Impossible => i18n
            .get_keys()
            .shared()
            .load_impossible_desc()
            .inner()
            .to_string(),
    }
}

#[component]
pub fn DailyLoadList(selected_load: RwSignal<DailyLoad>) -> impl IntoView {
    let i18n = use_i18n();

    view! {
        <div class="space-y-3">
            <For
                each=move || DAILY_LOAD_OPTIONS.iter().enumerate()
                key=|(idx, _)| *idx
                children=move |(_idx, load)| {
                    let load = *load;
                    let is_selected = Memo::new(move |_| selected_load.get() == load);
                    let load_for_click = load;
                    let load_code = format!("{:?}", load).to_lowercase();
                    let label = load_label(&i18n, &load);
                    let description = load_description(&i18n, &load);

                    view! {
                        <div
                            class=move || {
                                let base = "p-4 border cursor-pointer transition-all";
                                if is_selected.get() {
                                    format!("{} selected", base)
                                } else {
                                    base.to_string()
                                }
                            }
                            style=move || {
                                if is_selected.get() {
                                    "border: 2px solid var(--accent-olive); background: var(--bg-warm)"
                                } else {
                                    "border: 1px solid var(--border-dark)"
                                }
                            }
                            data-testid=format!("daily-load-option-{}", load_code)
                            on:click=move |_| {
                                selected_load.set(load_for_click);
                            }
                        >
                            <div class="flex items-center gap-3">
                                <div class="flex-1">
                                    <Text size=TextSize::Default variant=TypographyVariant::Primary>
                                        {label}
                                    </Text>
                                    <Text size=TextSize::Small variant=TypographyVariant::Muted>
                                        {description}
                                    </Text>
                                </div>
                                <div
                                    class="w-5 h-5 border relative flex-shrink-0 transition-all"
                                    style=move || {
                                        if is_selected.get() {
                                            "border: 1px solid var(--accent-olive)"
                                        } else {
                                            "border: 1px solid var(--border-dark)"
                                        }
                                    }
                                >
                                    {move || {
                                        if is_selected.get() {
                                            view! {
                                                <div
                                                    class="absolute"
                                                    style="inset: 3px; background: var(--fg-black)"
                                                ></div>
                                            }.into_any()
                                        } else {
                                            ().into_any()
                                        }
                                    }}
                                </div>
                            </div>
                        </div>
                    }
                }
            />
        </div>
    }
}
