use std::collections::HashMap;

use crate::i18n::*;
use crate::ui_components::{Checkbox, Text, TextSize, TypographyVariant};
use leptos::prelude::*;
use origa::domain::WellKnownSetMeta;

use super::onboarding_state::OnboardingState;

fn get_type_label(set_type: &str) -> String {
    match set_type {
        "Jlpt" => "JLPT".to_string(),
        "Migii" => "Migii".to_string(),
        "DuolingoRu" => "Duolingo 「RU」".to_string(),
        "DuolingoEn" => "Duolingo 「EN」".to_string(),
        "MinnaNoNihongo" => "Minna no Nihongo".to_string(),
        other => other.to_string(),
    }
}

fn get_type_icon(set_type: &str) -> Option<&'static str> {
    match set_type {
        "Migii" => Some("/public/external_icons/migii.png"),
        "DuolingoRu" => Some("/public/external_icons/duolingo.png"),
        "DuolingoEn" => Some("/public/external_icons/duolingo.png"),
        "MinnaNoNihongo" => Some("/public/external_icons/minnanonihongo.png"),
        _ => None,
    }
}

fn format_word_count(i18n: I18nContext<Locale>, count: usize) -> String {
    let summary = i18n.get_keys().onboarding().summary();
    if count == 0 {
        return summary.no_words_count().inner().to_string();
    }
    match i18n.get_locale() {
        Locale::ru => {
            let template = match count % 10 {
                1 if count % 100 != 11 => summary.word_single().inner(),
                2..=4 if !(12..=14).contains(&(count % 100)) => summary.word_dual().inner(),
                _ => summary.word_plural().inner(),
            };
            template.to_string().replace("{count}", &count.to_string())
        },
        Locale::en => {
            let template = if count == 1 {
                summary.word_single().inner()
            } else {
                summary.word_plural().inner()
            };
            template.to_string().replace("{count}", &count.to_string())
        },
    }
}

fn format_set_count(i18n: I18nContext<Locale>, count: usize) -> String {
    let summary = i18n.get_keys().onboarding().summary();
    match i18n.get_locale() {
        Locale::ru => {
            let template = match count % 10 {
                1 if count % 100 != 11 => summary.set_single().inner(),
                2..=4 if !(12..=14).contains(&(count % 100)) => summary.set_dual().inner(),
                _ => summary.set_plural().inner(),
            };
            template.to_string().replace("{count}", &count.to_string())
        },
        Locale::en => {
            let template = if count == 1 {
                summary.set_single().inner()
            } else {
                summary.set_plural().inner()
            };
            template.to_string().replace("{count}", &count.to_string())
        },
    }
}

#[component]
pub fn SummaryStep(#[prop(optional, into)] test_id: Signal<String>) -> impl IntoView {
    let i18n = use_i18n();
    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    let state =
        use_context::<RwSignal<OnboardingState>>().expect("OnboardingState context not found");

    let excluded_sets = Memo::new(move |_| state.get().excluded_sets.clone());

    let sets_by_type: Signal<HashMap<String, Vec<WellKnownSetMeta>>> = Signal::derive(move || {
        let sets = state.get().sets_to_import.clone();
        let mut result: HashMap<String, Vec<WellKnownSetMeta>> = HashMap::new();
        for set in sets {
            result.entry(set.set_type.clone()).or_default().push(set);
        }
        for sets in result.values_mut() {
            sets.sort_by(|a, b| a.title_ru.cmp(&b.title_ru));
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

    let i18n_for_summary = i18n;
    view! {
        <div class="summary-step" data-testid=test_id_val>
            <div class="text-center mb-6">
                <Text size=TextSize::Large variant=TypographyVariant::Primary test_id=Signal::derive(|| "summary-step-title".to_string())>
                    {t!(i18n, onboarding.summary.title)}
                </Text>
                <div class="mt-2">
                    <Text size=TextSize::Default variant=TypographyVariant::Muted test_id=Signal::derive(|| "summary-step-stats".to_string())>
                        {move || {
                            let count = total_count.get();
                            let word_count = total_word_count.get();
                            i18n_for_summary.get_keys().onboarding().summary().selected_count().inner().to_string()
                                .replace("{sets_count}", &count.to_string())
                                .replace("{words}", &format_word_count(i18n_for_summary, word_count))
                        }}
                    </Text>
                </div>
            </div>

            <div class="accordion-item-group">
                <For
                    each=move || sorted_types.get()
                    key=|t| t.clone()
                    children=move |set_type| {
                        let set_type_for_signal = set_type.clone();
                        let set_type_for_id_1 = set_type.clone();
                        let set_type_for_id_2 = set_type.clone();
                        let set_type_for_id_3 = set_type.clone();
                        let set_type_for_icon = set_type.clone();
                        let sets_for_type: Signal<Vec<WellKnownSetMeta>> = Signal::derive(
                            move || sets_by_type.get().get(&set_type_for_signal).cloned().unwrap_or_default(),
                        );
                        let type_word_count: Signal<usize> = Signal::derive(move || {
                            sets_for_type.get().iter().map(|s| s.word_count).sum()
                        });
                        let type_set_count: Signal<usize> = Signal::derive(move || {
                            sets_for_type.get().len()
                        });
                        let type_label = get_type_label(&set_type);
                        let type_label_for_img = type_label.clone();
                        let is_expanded = RwSignal::new(true);

                        view! {
                            <div class=move || {
                                format!("accordion-item {}", if is_expanded.get() { "active" } else { "" })
                            }>
                                <div
                                    class="accordion-header"
                                    data-testid="summary-step-accordion-header"
                                    on:click=move |_| is_expanded.update(|v| *v = !*v)
                                >
                                    <div class="flex items-center gap-2">
                                        {move || {
                                            let icon = get_type_icon(&set_type_for_icon);
                                            if let Some(icon_path) = icon {
                                                view! {
                                                    <img src=icon_path class="w-6 h-6 object-contain" alt=type_label_for_img.clone() />
                                                }.into_any()
                                            } else {
                                                ().into_any()
                                            }
                                        }}
                                        <Text size=TextSize::Default variant=TypographyVariant::Primary test_id=Signal::derive(move || format!("summary-step-type-{}", set_type_for_id_1.clone()))>
                                            {type_label}
                                        </Text>
                                        <Text size=TextSize::Small variant=TypographyVariant::Muted test_id=Signal::derive(move || format!("summary-step-type-count-{}", set_type_for_id_2.clone()))>
                                            {move || format!("({})", format_set_count(i18n, type_set_count.get()))}
                                        </Text>
                                    </div>
                                    <div class="flex items-center gap-3">
                                        <Text size=TextSize::Small variant=TypographyVariant::Muted test_id=Signal::derive(move || format!("summary-step-words-{}", set_type_for_id_3.clone()))>
                                            {move || format_word_count(i18n, type_word_count.get())}
                                        </Text>
                                        <div class="accordion-icon"></div>
                                    </div>
                                </div>
                                <div
                                    class="accordion-content"
                                    style:max-height=move || {
                                        let count = sets_for_type.get().len();
                                        let estimated_height = (count * 44 + 40).max(200);
                                        if is_expanded.get() {
                                            format!("{}px", estimated_height)
                                        } else {
                                            "0px".to_string()
                                        }
                                    }
                                >
                                    <div class="accordion-body">
                                        <For
                                            each=move || sets_for_type.get()
                                            key=|s| s.id.clone()
                                            children=move |set_meta| {
                                                let set_meta_id = set_meta.id.clone();
                                                let set_id_for_cb = set_meta_id.clone();
                                                let set_id_for_memo = set_meta_id.clone();
                                                let set_title = set_meta.title_ru.clone();
                                                let word_count = set_meta.word_count;
                                                let is_excluded =
                                                    Memo::new(move |_| excluded_sets.get().contains(&set_id_for_memo));
                                                let set_test_id = format!("summary-step-set-{}", set_meta_id);
                                                let set_test_id_1 = set_test_id.clone();
                                                let set_test_id_2 = set_test_id.clone();
                                                let set_test_id_3 = set_test_id.clone();

                                                view! {
                                                    <div class="checkbox-container py-2">
                                                        <Checkbox
                                                            checked=Signal::derive(move || !is_excluded.get())
                                                            label=Signal::derive(String::new)
                                                            on_change=Callback::new(move |()| {
                                                                toggle_set.run(set_id_for_cb.clone());
                                                            })
                                                            test_id=Signal::derive(move || format!("{}-checkbox", set_test_id_1.clone()))
                                                        />
                                                        <span class="flex-1">
                                                            <Text size=TextSize::Small variant=TypographyVariant::Primary test_id=Signal::derive(move || format!("{}-title", set_test_id_2.clone()))>
                                                                {set_title}
                                                            </Text>
                                                        </span>
                                                        <Text size=TextSize::Small variant=TypographyVariant::Muted test_id=Signal::derive(move || format!("{}-words", set_test_id_3.clone()))>
                                                            {format_word_count(i18n, word_count)}
                                                        </Text>
                                                    </div>
                                                }
                                            }
                                        />
                                    </div>
                                </div>
                            </div>
                        }
                    }
                />
            </div>

            <Show when=move || total_count.get() == 0>
                <div class="text-center py-8">
                    <Text size=TextSize::Default variant=TypographyVariant::Muted test_id=Signal::derive(|| "summary-step-empty".to_string())>
                        {t!(i18n, onboarding.summary.nothing_selected)}
                    </Text>
                    <div class="mt-2">
                        <Text size=TextSize::Small variant=TypographyVariant::Muted test_id=Signal::derive(|| "summary-step-empty-hint".to_string())>
                            {t!(i18n, onboarding.summary.nothing_hint)}
                        </Text>
                    </div>
                </div>
            </Show>
        </div>
    }
}
