use crate::ui_components::{Text, TextSize, TypographyVariant};
use leptos::prelude::*;
use origa::domain::DailyLoad;

#[component]
pub fn DailyLoadSelector(selected_load: RwSignal<DailyLoad>) -> impl IntoView {
    view! {
        <div class="space-y-2" data-testid="daily-load-selector">
            <Text size=TextSize::Small variant=TypographyVariant::Muted>
                "Темп обучения"
            </Text>
            <div class="grid grid-cols-2 gap-2">
                {DailyLoad::all().iter().map(|load| {
                    let load_val = *load;
                    let is_selected = move || selected_load.get() == load_val;
                    let load_for_cb = load_val;
                    view! {
                        <button
                            class=move || {
                                let base = "p-2 border rounded-lg text-left transition-all cursor-pointer text-sm";
                                if is_selected() {
                                    format!("{} border-[var(--accent-olive)] bg-[var(--accent-olive)]/10", base)
                                } else {
                                    format!("{} border-[var(--border-dark)] hover:border-[var(--accent-olive)]/50", base)
                                }
                            }
                            on:click=move |_| selected_load.set(load_for_cb)
                        >
                            <div class="font-medium">
                                {load_val.as_str().to_string()}
                            </div>
                            <div class="text-xs text-[var(--fg-muted)] mt-0.5">
                                {load_val.description().to_string()}
                            </div>
                        </button>
                    }
                }).collect::<Vec<_>>()
                }
            </div>
        </div>
    }
}
