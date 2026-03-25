use crate::ui_components::{Card, Dropdown, DropdownItem, Text, TextSize, TypographyVariant};
use leptos::prelude::*;

use super::super::onboarding_state::OnboardingState;
use super::types::MinnaLesson;

#[component]
pub fn MinnaProgressSelector(
    app_id: String,
    title: String,
    lessons: Vec<MinnaLesson>,
    state: RwSignal<OnboardingState>,
) -> impl IntoView {
    let selected_lesson = RwSignal::new(None::<usize>);
    let available_sets = Signal::derive(move || state.get().available_sets.clone());

    let lesson_items = {
        let mut items = vec![DropdownItem {
            value: "none".to_string(),
            label: "Не изучал".to_string(),
        }];
        for lesson in &lessons {
            items.push(DropdownItem {
                value: format!("lesson_{}", lesson.lesson_number),
                label: format!("Урок {}", lesson.lesson_number),
            });
        }
        items
    };

    let import_info = Signal::derive(move || {
        selected_lesson
            .get()
            .map(|n| format!("Будут импортированы: Уроки 1-{}", n))
    });

    let lessons_for_effect = lessons.clone();
    let app_id_for_effect = app_id.clone();
    Effect::new(move |_| {
        let lesson_num = selected_lesson.get();
        let lessons_ref = lessons_for_effect.clone();
        let sets = available_sets.get();

        if let Some(n) = lesson_num {
            let ids_to_import: Vec<String> = lessons_ref
                .iter()
                .filter(|l| l.lesson_number <= n)
                .map(|l| l.id.clone())
                .collect();

            let aid = app_id_for_effect.clone();
            state.update(|s| {
                s.set_app_selection(&aid, &format!("lesson_{}", n));
                s.sets_to_import
                    .retain(|set| !lessons_ref.iter().any(|l| l.id == set.id));
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

    let selected_lesson_value = RwSignal::new(
        selected_lesson
            .get()
            .map(|n| format!("lesson_{}", n))
            .unwrap_or_else(|| "none".to_string()),
    );

    Effect::new(move |_| {
        let val = selected_lesson_value.get();
        selected_lesson.set(
            val.strip_prefix("lesson_")
                .and_then(|s| s.parse::<usize>().ok()),
        );
    });

    let title_for_view = title.clone();
    view! {
        <Card class=Signal::derive(|| "p-4".to_string())>
            <Text size=TextSize::Default variant=TypographyVariant::Primary>
                {title_for_view}
            </Text>

            <div class="mt-4">
                <Text size=TextSize::Small variant=TypographyVariant::Muted>
                    "Урок"
                </Text>
                <div class="mt-2">
                    <Dropdown
                        _options=Signal::derive(move || lesson_items.clone())
                        _selected=selected_lesson_value
                        _placeholder=Signal::derive(|| "Выберите урок".to_string())
                        test_id=Signal::derive(move || format!("{}-lesson-dropdown", app_id.clone()))
                    />
                </div>
            </div>

            <Show when=move || import_info.get().is_some()>
                <div class="mt-2">
                    <Text size=TextSize::Small variant=TypographyVariant::Muted>
                        {move || import_info.get().unwrap_or_default()}
                    </Text>
                </div>
            </Show>
        </Card>
    }
}
