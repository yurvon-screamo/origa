use std::collections::HashMap;

use crate::core::config::public_url;
use crate::i18n::{t, use_i18n};
use crate::ui_components::{Card, Dropdown, Text, TextSize, TypographyVariant};
use leptos::prelude::*;
use origa::domain::JapaneseLevel;

use super::super::onboarding_state::OnboardingState;
use super::import_info::build_cumulative_import_info;
use super::minna_helpers::{
    build_lesson_items, build_level_items, collect_extras_to_import, collect_lessons_to_import,
    is_extra_in_levels, is_lesson_in_levels,
};
use super::types::MinnaLesson;

fn parse_level(val: &str) -> Option<JapaneseLevel> {
    match val {
        "N5" => Some(JapaneseLevel::N5),
        "N4" => Some(JapaneseLevel::N4),
        "N3" => Some(JapaneseLevel::N3),
        "N2" => Some(JapaneseLevel::N2),
        "N1" => Some(JapaneseLevel::N1),
        _ => None,
    }
}

#[component]
pub fn MinnaProgressSelector(
    lessons_by_level: Signal<HashMap<JapaneseLevel, Vec<MinnaLesson>>>,
    extras_by_level: Signal<HashMap<JapaneseLevel, Vec<String>>>,
    state: RwSignal<OnboardingState>,
) -> impl IntoView {
    let i18n = use_i18n();
    let selected_level = RwSignal::new("none".to_string());
    let selected_lesson = RwSignal::new("none".to_string());
    let available_sets = Signal::derive(move || state.get().available_sets.clone());

    let level_items = Signal::derive(move || build_level_items(&i18n, &lessons_by_level.get()));

    let parsed_level = Signal::derive(move || parse_level(&selected_level.get()));

    let lesson_items = Signal::derive(move || {
        build_lesson_items(
            &i18n,
            &lessons_by_level.get(),
            &extras_by_level.get(),
            parsed_level.get(),
        )
    });

    let import_info = Signal::derive(move || {
        let level = parsed_level.get();
        let selection = selected_lesson.get();

        if selection == "extra" {
            return level.map(|lvl| {
                i18n.get_keys()
                    .onboarding()
                    .progress()
                    .import_extra()
                    .inner()
                    .to_string()
                    .replacen("{}", super::app_type::level_to_str(lvl), 1)
            });
        }

        let lesson_num = selection
            .strip_prefix("lesson_")
            .and_then(|s| s.parse::<usize>().ok());

        build_cumulative_import_info(&i18n, level, lesson_num)
    });

    Effect::new(move |_| {
        let level = parsed_level.get();
        let selection = selected_lesson.get();

        let Some(lvl) = level else {
            return;
        };

        let lessons_by_level_snapshot = lessons_by_level.get_untracked();
        let extras_by_level_snapshot = extras_by_level.get_untracked();
        let sets_snapshot: Vec<_> = available_sets.get_untracked();

        let ids_to_import = if selection == "extra" {
            collect_extras_to_import(&extras_by_level_snapshot, lvl)
        } else {
            let Some(lesson_n) = selection
                .strip_prefix("lesson_")
                .and_then(|s| s.parse::<usize>().ok())
            else {
                return;
            };
            collect_lessons_to_import(&lessons_by_level_snapshot, lvl, lesson_n)
        };

        let selection_key = if selection == "extra" {
            format!("{:?}_extra", lvl)
        } else {
            selection.clone()
        };

        state.update(|s| {
            s.set_app_selection("MinnaNoNihongo", &selection_key);
            s.sets_to_import.retain(|set| {
                !is_lesson_in_levels(set.id.as_str(), &lessons_by_level_snapshot)
                    && !is_extra_in_levels(set.id.as_str(), &extras_by_level_snapshot)
            });
            let sets_to_add: Vec<_> = sets_snapshot
                .iter()
                .filter(|set_meta| ids_to_import.contains(&set_meta.id))
                .cloned()
                .collect();
            for set_meta in sets_to_add {
                s.add_set_to_import(set_meta);
            }
        });
    });

    view! {
        <Card class=Signal::derive(|| "p-4".to_string())>
            <div class="flex items-center gap-3 mb-2">
                <img
                    src=public_url("/public/external_icons/minnanonihongo.png")
                    class="w-12 h-12 object-contain"
                    alt="Minna no Nihongo"
                />
                <Text size=TextSize::Default variant=TypographyVariant::Primary>
                    "Minna no Nihongo"
                </Text>
            </div>

            <div class="mt-4 space-y-4">
                <div>
                    <Text size=TextSize::Small variant=TypographyVariant::Muted>
                        {t!(i18n, onboarding.progress.level)}
                    </Text>
                    <div class="mt-2">
                        <Dropdown
                            options=level_items
                            selected=selected_level
                            placeholder=Signal::derive(move || i18n.get_keys().onboarding().progress().select_level().inner().to_string())
                            test_id=Signal::derive(|| "minna-level-dropdown".to_string())
                        />
                    </div>
                </div>

                <Show when=move || parsed_level.get().is_some()>
                    <div>
                        <Text size=TextSize::Small variant=TypographyVariant::Muted>
                            {t!(i18n, onboarding.progress.lesson)}
                        </Text>
                        <div class="mt-2">
                            <Dropdown
                                options=lesson_items
                                selected=selected_lesson
                                placeholder=Signal::derive(move || i18n.get_keys().onboarding().progress().select_lesson().inner().to_string())
                                test_id=Signal::derive(|| "minna-lesson-dropdown".to_string())
                            />
                        </div>
                    </div>
                </Show>

                <Show when=move || import_info.get().is_some()>
                    <div class="mt-2">
                        <Text size=TextSize::Small variant=TypographyVariant::Muted>
                            {move || import_info.get().unwrap_or_default()}
                        </Text>
                    </div>
                </Show>
            </div>
        </Card>
    }
}
