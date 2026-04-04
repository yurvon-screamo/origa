use crate::ui_components::{Text, TextSize, TypographyVariant};
use leptos::prelude::*;
use origa::domain::DailyLoad;

const DAILY_LOAD_OPTIONS: &[(DailyLoad, &str, &str)] = &[
    (
        DailyLoad::Light,
        "Лёгкий",
        "Лёгкая нагрузка для комфортного, но медленного обучения. Будьте готовы уделять 10-15 минут в день.",
    ),
    (
        DailyLoad::Medium,
        "Сбалансированный",
        "Сбалансированный и комфортный темп обучения. Будьте готовы уделять 25-40 минут в день.",
    ),
    (
        DailyLoad::Hard,
        "Сложный",
        "Интенсивное обучение для быстрого прогресса. Будьте готовы уделять 1-2 часа в день.",
    ),
    (
        DailyLoad::Impossible,
        "Максимальный",
        "Высочайший уровень нагрузки для достижения максимального прогресса. Будьте готовы уделять 3-5 часов в день.",
    ),
];

#[component]
pub fn DailyLoadList(selected_load: RwSignal<DailyLoad>) -> impl IntoView {
    view! {
        <div class="space-y-3">
            <For
                each=move || DAILY_LOAD_OPTIONS.iter().enumerate()
                key=|(idx, _)| *idx
                children=move |(_idx, (load, label, description))| {
                    let load = *load;
                    let label = *label;
                    let description = *description;
                    let is_selected = Memo::new(move |_| selected_load.get() == load);
                    let load_for_click = load;
                    let load_code = format!("{:?}", load).to_lowercase();

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
