use crate::ui_components::{Text, TextSize, TypographyVariant};
use leptos::prelude::*;
use origa::dictionary::RadicalInfo;
use std::collections::HashSet;

#[component]
pub fn RadicalList(
    radical_list: Vec<RadicalInfo>,
    selected_radicals: RwSignal<HashSet<char>>,
    known_radicals: HashSet<char>,
) -> impl IntoView {
    if radical_list.is_empty() {
        return view! {
            <Text size=TextSize::Small variant=TypographyVariant::Muted>
                "Нет радикалов для выбранного уровня (или все уже изучены)"
            </Text>
        }
        .into_any();
    }

    view! {
        <div class="space-y-2 overflow-y-auto">
            <For
                each=move || radical_list.clone()
                key=|radical| radical.radical()
                children=move |radical_info| {
                    view! {
                        <RadicalItemDisplay
                            radical_info=radical_info
                            selected_radicals=selected_radicals
                            known_radicals=known_radicals.clone()
                        />
                    }
                }
            />
        </div>
    }
    .into_any()
}

#[component]
fn RadicalItemDisplay(
    radical_info: RadicalInfo,
    selected_radicals: RwSignal<HashSet<char>>,
    known_radicals: HashSet<char>,
) -> impl IntoView {
    let radical_char = radical_info.radical();
    let is_selected = Memo::new(move |_| selected_radicals.get().contains(&radical_char));
    let is_known = known_radicals.contains(&radical_char);

    let name = radical_info.name().to_string();
    let description = radical_info.description().to_string();
    let stroke_count = radical_info.stroke_count();

    view! {
        <div
            class=Signal::derive(move || {
                format!(
                    "p-3 border cursor-pointer transition-all {}",
                    if is_selected.get() { "border-[var(--accent-olive)] bg-[var(--bg-aged)]" } else { "border-[var(--border-dark)] bg-[var(--bg-paper)]" }
                )
            })
            on:click={
                move |_| {
                    selected_radicals.update(|set| {
                        if set.contains(&radical_char) {
                            set.remove(&radical_char);
                        } else {
                            set.insert(radical_char);
                        }
                    });
                }
            }
        >
            <div class="flex items-center gap-3">
                <span class="text-2xl font-serif">{radical_char}</span>
                <div class="flex-1">
                    <Text size=TextSize::Small variant=TypographyVariant::Primary>
                        {name}
                    </Text>
                    <Text size=TextSize::Small variant=TypographyVariant::Muted>
                        {format!("{}, {} черт", description, stroke_count)}
                    </Text>
                </div>
                {move || {
                    if is_known {
                        view! {
                            <Text size=TextSize::Small variant=TypographyVariant::Muted>
                                "✓"
                            </Text>
                        }.into_any()
                    } else {
                        ().into_any()
                    }
                }}
            </div>
        </div>
    }
}
