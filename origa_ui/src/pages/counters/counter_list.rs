use super::counter_item::CounterItem;
use crate::i18n::{t, use_i18n};
use crate::ui_components::{Text, TextSize, TypographyVariant};
use leptos::prelude::*;
use origa::dictionary::counters::CounterEntry;
use origa::domain::NativeLanguage;
use std::collections::HashSet;

#[component]
pub fn CounterList(
    counter_list: Vec<&'static CounterEntry>,
    #[prop(into)] native_language: Signal<NativeLanguage>,
    selected_counters: RwSignal<HashSet<String>>,
    known_counters: HashSet<String>,
) -> impl IntoView {
    let i18n = use_i18n();
    if counter_list.is_empty() {
        return view! {
            <div data-testid="counters-drawer-empty">
                <Text size=TextSize::Small variant=TypographyVariant::Muted>
                    {t!(i18n, counters.no_counters_for_level)}
                </Text>
            </div>
        }
        .into_any();
    }

    view! {
        <div class="counter-grid overflow-y-auto">
            <For
                each=move || counter_list.clone()
                key=|entry| entry.suffix().to_string()
                children=move |counter_entry| {
                    view! {
                        <CounterItem
                            counter_entry=counter_entry
                            native_language=native_language
                            selected_counters=selected_counters
                            known_counters=known_counters.clone()
                        />
                    }
                }
            />
        </div>
    }
    .into_any()
}
