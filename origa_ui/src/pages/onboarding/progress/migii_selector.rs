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
pub fn MigiiProgressSelector(
    lessons_by_level: Signal<HashMap<JapaneseLevel, Vec<MigiiLesson>>>,
    state: RwSignal<OnboardingState>,
) -> impl IntoView {
    let selected_level = RwSignal::new("none".to_string());
    let selected_lesson = RwSignal::new("none".to_string());
    let available_sets = Signal::derive(move || state.get().available_sets.clone());

    let level_items = build_level_items();

    let parsed_level = Signal::derive(move || parse_level(&selected_level.get()));

    let lesson_items =
        Signal::derive(move || build_lesson_items(&lessons_by_level.get(), parsed_level.get()));

    let import_info = Signal::derive(move || {
        let level = parsed_level.get();
        let lesson = selected_lesson
            .get()
            .strip_prefix("lesson_")
            .and_then(|s| s.parse::<usize>().ok());

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

    Effect::new(move |_| {
        let level = parsed_level.get();
        let lesson_num = selected_lesson
            .get()
            .strip_prefix("lesson_")
            .and_then(|s| s.parse::<usize>().ok());

        if level.is_none() || lesson_num.is_none() {
            return;
        }

        web_sys::console::log_1(&"[Migii] Effect START".into());

        let lessons_by_snapshot = lessons_by_level.get_untracked();
        let sets_snapshot: Vec<_> = available_sets.get_untracked();

        if let (Some(lvl), Some(lesson_n)) = (level, lesson_num)
            && let Some(lessons) = lessons_by_snapshot.get(&lvl)
        {
            web_sys::console::log_1(&format!("[Migii] Processing level {:?}, lesson {}", lvl, lesson_n).into());
            let ids_to_import = collect_lessons_to_import(lessons, lesson_n);
            web_sys::console::log_1(&format!("[Migii] ids_to_import count: {}", ids_to_import.len()).into());

            state.update(|s| {
                web_sys::console::log_1(&"[Migii] state.update START".into());
                s.set_app_selection("Migii", &format!("{:?}_{}", lvl, lesson_n));
                s.sets_to_import
                    .retain(|set| !is_lesson_in_levels(set.id.as_str(), &lessons_by_snapshot));
                let sets_to_add: Vec<_> = sets_snapshot
                    .iter()
                    .filter(|set_meta| ids_to_import.contains(&set_meta.id))
                    .cloned()
                    .collect();
                for set_meta in sets_to_add {
                    s.add_set_to_import(set_meta);
                }
                web_sys::console::log_1(&"[Migii] state.update END".into());
            });
        }
        web_sys::console::log_1(&"[Migii] Effect END".into());
    });

    view! {
        <Card class=Signal::derive(|| "p-4".to_string())>
            <div class="flex items-center gap-3 mb-2">
                <img src="/public/external_icons/migii.png" class="w-12 h-12 object-contain" alt="Migii" />
                <Text size=TextSize::Default variant=TypographyVariant::Primary>
                    "Migii"
                </Text>
            </div>

            <div class="mt-4 space-y-4">
                <div>
                    <Text size=TextSize::Small variant=TypographyVariant::Muted>
                        "Уровень"
                    </Text>
                    <div class="mt-2">
                        <Dropdown
                            _options=Signal::derive(move || level_items.clone())
                            _selected=selected_level
                            _placeholder=Signal::derive(|| "Выберите уровень".to_string())
                            test_id=Signal::derive(|| "migii-level-dropdown".to_string())
                        />
                    </div>
                </div>

                <Show when=move || parsed_level.get().is_some()>
                    <div>
                        <Text size=TextSize::Small variant=TypographyVariant::Muted>
                            "Урок"
                        </Text>
                    <div class="mt-2">
                        <Dropdown
                            _options=lesson_items
                            _selected=selected_lesson
                            _placeholder=Signal::derive(|| "Выберите урок".to_string())
                            test_id=Signal::derive(|| "migii-lesson-dropdown".to_string())
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
