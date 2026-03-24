use std::collections::HashMap;

use crate::ui_components::{Card, Dropdown, Text, TextSize, TypographyVariant};
use leptos::prelude::*;
use origa::domain::JapaneseLevel;

use super::super::onboarding_state::OnboardingState;
use super::app_type::level_to_str;
use super::migii_helpers::{
    build_lesson_items, build_level_items, collect_lessons_to_import, is_lesson_in_levels,
};
use super::types::MigiiLesson;

#[component]
pub fn MigiiProgressSelector(
    lessons_by_level: HashMap<JapaneseLevel, Vec<MigiiLesson>>,
    state: RwSignal<OnboardingState>,
) -> impl IntoView {
    let selected_level = RwSignal::new(None::<JapaneseLevel>);
    let selected_lesson = RwSignal::new(None::<usize>);
    let available_sets = Signal::derive(move || state.get().available_sets.clone());

    let level_items = build_level_items();

    let lessons_by_level_for_items = lessons_by_level.clone();
    let lesson_items = Signal::derive(move || {
        build_lesson_items(&lessons_by_level_for_items, selected_level.get())
    });

    let import_info = Signal::derive(move || {
        let level = selected_level.get();
        let lesson = selected_lesson.get();

        match (level, lesson) {
            (Some(lvl), Some(n)) => Some(format!(
                "Будут импортированы: {} Уроки 1-{}",
                level_to_str(lvl),
                n
            )),
            (Some(lvl), None) => Some(format!("Выберите урок для {}", level_to_str(lvl))),
            _ => None,
        }
    });

    let lessons_by_level_for_effect = lessons_by_level.clone();
    Effect::new(move |_| {
        let level = selected_level.get();
        let lesson_num = selected_lesson.get();
        let lessons_by = lessons_by_level_for_effect.clone();
        let sets = available_sets.get();

        if let (Some(lvl), Some(lesson_n)) = (level, lesson_num)
            && let Some(lessons) = lessons_by.get(&lvl)
        {
            let ids_to_import = collect_lessons_to_import(lessons, lesson_n);

            state.update(|s| {
                s.set_app_selection("Migii", &format!("{:?}_{}", lvl, lesson_n));
                s.sets_to_import
                    .retain(|set| !is_lesson_in_levels(set.id.as_str(), &lessons_by));
                let sets_to_add: Vec<_> = sets
                    .iter()
                    .filter(|set_meta| ids_to_import.contains(&set_meta.id))
                    .cloned()
                    .collect();
                for set_meta in sets_to_add {
                    s.add_set_to_import(set_meta);
                }
            });
        }
    });

    let selected_level_value = RwSignal::new(
        selected_level
            .get()
            .map(|l| level_to_str(l).to_string())
            .unwrap_or_else(|| "none".to_string()),
    );
    let selected_lesson_value = RwSignal::new(
        selected_lesson
            .get()
            .map(|n| format!("lesson_{}", n))
            .unwrap_or_else(|| "none".to_string()),
    );

    Effect::new(move |_| {
        let val = selected_level_value.get();
        selected_level.set(match val.as_str() {
            "N5" => Some(JapaneseLevel::N5),
            "N4" => Some(JapaneseLevel::N4),
            "N3" => Some(JapaneseLevel::N3),
            "N2" => Some(JapaneseLevel::N2),
            "N1" => Some(JapaneseLevel::N1),
            _ => None,
        });
    });

    Effect::new(move |_| {
        let val = selected_lesson_value.get();
        selected_lesson.set(
            val.strip_prefix("lesson_")
                .and_then(|s| s.parse::<usize>().ok()),
        );
    });

    view! {
        <Card class=Signal::derive(|| "p-4".to_string())>
            <Text size=TextSize::Default variant=TypographyVariant::Primary>
                "Migii"
            </Text>

            <div class="mt-4 space-y-4">
                <div>
                    <Text size=TextSize::Small variant=TypographyVariant::Muted>
                        "Уровень"
                    </Text>
                    <div class="mt-2">
                        <Dropdown
                            _options=Signal::derive(move || level_items.clone())
                            _selected=selected_level_value
                            _placeholder=Signal::derive(|| "Выберите уровень".to_string())
                        />
                    </div>
                </div>

                <Show when=move || selected_level.get().is_some()>
                    <div>
                        <Text size=TextSize::Small variant=TypographyVariant::Muted>
                            "Урок"
                        </Text>
                        <div class="mt-2">
                            <Dropdown
                                _options=lesson_items
                                _selected=selected_lesson_value
                                _placeholder=Signal::derive(|| "Выберите урок".to_string())
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
