use std::collections::HashMap;

use crate::ui_components::{Checkbox, Text, TextSize, TypographyVariant};
use leptos::prelude::*;
use origa::traits::WellKnownSetMeta;

use super::onboarding_state::OnboardingState;

fn get_type_label(set_type: &str) -> String {
    match set_type {
        "Jlpt" => "JLPT".to_string(),
        "Migii" => "Migii".to_string(),
        "DuolingoRu" => "Duolingo (RU)".to_string(),
        "DuolingoEn" => "Duolingo (EN)".to_string(),
        "MinnaNoNihongo" => "Minna no Nihongo".to_string(),
        other => other.to_string(),
    }
}

fn format_word_count(count: usize) -> String {
    match count {
        0 => "0 слов".to_string(),
        1 => "1 слово".to_string(),
        n if n < 5 => format!("{} слова", n),
        n if n < 20 => format!("{} слов", n),
        n if n % 10 == 1 => format!("{} слово", n),
        n if n % 10 < 5 => format!("{} слова", n),
        n => format!("{} слов", n),
    }
}

#[component]
pub fn SummaryStep() -> impl IntoView {
    let state =
        use_context::<RwSignal<OnboardingState>>().expect("OnboardingState context not found");

    let excluded_sets = Memo::new(move |_| state.get().excluded_sets.clone());

    let sets_by_type: Signal<HashMap<String, Vec<WellKnownSetMeta>>> = Signal::derive(move || {
        let sets = state.get().sets_to_import.clone();
        let mut result: HashMap<String, Vec<WellKnownSetMeta>> = HashMap::new();
        for set in sets {
            result.entry(set.set_type.clone()).or_default().push(set);
        }
        result
    });

    let total_word_count: Signal<usize> = Signal::derive(move || {
        state
            .get()
            .sets_to_import
            .iter()
            .map(|s| s.word_count)
            .sum()
    });

    let total_count: Signal<usize> = Signal::derive(move || state.get().sets_to_import.len());

    let toggle_set = Callback::new(move |set_id: String| {
        let excluded = state.get().excluded_sets.contains(&set_id);

        state.update(|s| {
            if excluded {
                s.reset_exclusion(&set_id);
                if let Some(set_meta) = s.available_sets.iter().find(|set| set.id == set_id) {
                    s.add_set_to_import(set_meta.clone());
                }
            } else {
                s.remove_set_from_import(&set_id);
            }
        });
    });

    let sorted_types: Signal<Vec<String>> = Signal::derive(move || {
        let mut types: Vec<String> = sets_by_type.get().keys().cloned().collect();
        types.sort();
        types
    });

    view! {
        <div class="summary-step">
            <div class="text-center mb-6">
                <Text size=TextSize::Large variant=TypographyVariant::Primary>
                    "Готово к импорту"
                </Text>
                <div class="mt-2">
                    <Text size=TextSize::Default variant=TypographyVariant::Muted>
                        {move || {
                            let count = total_count.get();
                            let word_count = total_word_count.get();
                            format!("Выбрано {} наборов, {}", count, format_word_count(word_count))
                        }}
                    </Text>
                </div>
            </div>

            <div class="space-y-4">
                <For
                    each=move || sorted_types.get()
                    key=|t| t.clone()
                    children=move |set_type| {
                        let set_type_for_signal = set_type.clone();
                        let sets_for_type: Signal<Vec<WellKnownSetMeta>> = Signal::derive(
                            move || sets_by_type.get().get(&set_type_for_signal).cloned().unwrap_or_default(),
                        );
                        let type_word_count: Signal<usize> = Signal::derive(move || {
                            sets_for_type.get().iter().map(|s| s.word_count).sum()
                        });
                        let type_label = get_type_label(&set_type);

                        view! {
                            <div class="border rounded-lg p-4">
                                <div class="flex justify-between items-center mb-2">
                                    <Text size=TextSize::Default variant=TypographyVariant::Primary>
                                        {type_label}
                                    </Text>
                                    <Text size=TextSize::Small variant=TypographyVariant::Muted>
                                        {move || format_word_count(type_word_count.get())}
                                    </Text>
                                </div>

                                <div class="space-y-2">
                                    <For
                                        each=move || sets_for_type.get()
                                        key=|s| s.id.clone()
                                        children=move |set_meta| {
                                            let set_id = set_meta.id.clone();
                                            let set_id_for_cb = set_id.clone();
                                            let set_title = set_meta.title_ru.clone();
                                            let word_count = set_meta.word_count;
                                            let is_excluded =
                                                Memo::new(move |_| excluded_sets.get().contains(&set_id));

                                            view! {
                                                <div class="flex items-center gap-2 p-2 rounded hover:bg-gray-50">
                                                    <Checkbox
                                                        checked=Signal::derive(move || !is_excluded.get())
                                                        label=Signal::derive(String::new)
                                                        on_change=Callback::new(move |()| {
                                                            toggle_set.run(set_id_for_cb.clone());
                                                        })
                                                    />
                                                    <div class="flex-1">
                                                        <Text size=TextSize::Small variant=TypographyVariant::Primary>
                                                            {set_title}
                                                        </Text>
                                                    </div>
                                                    <Text size=TextSize::Small variant=TypographyVariant::Muted>
                                                        {format_word_count(word_count)}
                                                    </Text>
                                                </div>
                                            }
                                        }
                                    />
                                </div>
                            </div>
                        }
                    }
                />
            </div>

            <Show when=move || total_count.get() == 0>
                <div class="text-center py-8">
                    <Text size=TextSize::Default variant=TypographyVariant::Muted>
                        "Ничего не выбрано для импорта"
                    </Text>
                    <div class="mt-2">
                        <Text size=TextSize::Small variant=TypographyVariant::Muted>
                            "Вернитесь на предыдущие шаги, чтобы выбрать наборы"
                        </Text>
                    </div>
                </div>
            </Show>
        </div>
    }
}
