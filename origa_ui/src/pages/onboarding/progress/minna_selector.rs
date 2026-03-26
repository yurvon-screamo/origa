use crate::ui_components::{Card, Dropdown, DropdownItem, Text, TextSize, TypographyVariant};
use leptos::prelude::*;

use super::super::onboarding_state::OnboardingState;
use super::types::MinnaLesson;

#[component]
pub fn MinnaProgressSelector(
    app_id: String,
    title: String,
    lessons: Signal<Vec<MinnaLesson>>,
    state: RwSignal<OnboardingState>,
) -> impl IntoView {
    let selected_lesson = RwSignal::new("none".to_string());
    let available_sets = Signal::derive(move || state.get().available_sets.clone());

    let lesson_items = Signal::derive(move || {
        let mut items = vec![DropdownItem {
            value: "none".to_string(),
            label: "Не изучал".to_string(),
        }];
        for lesson in lessons.get().iter() {
            items.push(DropdownItem {
                value: format!("lesson_{}", lesson.lesson_number),
                label: format!("Урок {}", lesson.lesson_number),
            });
        }
        items
    });

    let import_info = Signal::derive(move || {
        selected_lesson
            .get()
            .strip_prefix("lesson_")
            .and_then(|s| s.parse::<usize>().ok())
            .map(|n| format!("Будут импортированы: Уроки 1-{}", n))
    });

    let app_id_for_effect = app_id.clone();
    Effect::new(move |_| {
        let val = selected_lesson.get();

        if val == "none" {
            return;
        }

        web_sys::console::log_1(&"[Minna] Effect START".into());

        let lesson_num = val
            .strip_prefix("lesson_")
            .and_then(|s| s.parse::<usize>().ok());

        if lesson_num.is_none() {
            web_sys::console::log_1(&"[Minna] lesson_num is None, returning".into());
            return;
        }

        let lessons_snapshot: Vec<_> = lessons.get_untracked();
        let sets_snapshot: Vec<_> = available_sets.get_untracked();

        if let Some(n) = lesson_num {
            web_sys::console::log_1(&format!("[Minna] Processing lesson_num: {}", n).into());
            let ids_to_import: Vec<String> = lessons_snapshot
                .iter()
                .filter(|l| l.lesson_number <= n)
                .map(|l| l.id.clone())
                .collect();

            web_sys::console::log_1(
                &format!("[Minna] ids_to_import count: {}", ids_to_import.len()).into(),
            );
            let aid = app_id_for_effect.clone();
            state.update(|s| {
                web_sys::console::log_1(&"[Minna] state.update START".into());
                s.set_app_selection(&aid, &format!("lesson_{}", n));
                s.sets_to_import
                    .retain(|set| !lessons_snapshot.iter().any(|l| l.id == set.id));
                let sets_to_add: Vec<_> = sets_snapshot
                    .iter()
                    .filter(|set_meta| ids_to_import.contains(&set_meta.id))
                    .cloned()
                    .collect();
                for set_meta in sets_to_add {
                    s.add_set_to_import(set_meta);
                }
                web_sys::console::log_1(&"[Minna] state.update END".into());
            });
        }
        web_sys::console::log_1(&"[Minna] Effect END".into());
    });

    let title_for_view = title.clone();
    let app_id_for_dropdown = app_id.clone();
    view! {
        <Card class=Signal::derive(|| "p-4".to_string())>
            <div class="flex items-center gap-3 mb-2">
                <img src="/public/external_icons/minnanonihongo.png" class="w-12 h-12 object-contain" alt="Minna no Nihongo" />
                <Text size=TextSize::Default variant=TypographyVariant::Primary>
                    {title_for_view}
                </Text>
            </div>

            <div class="mt-4">
                <Text size=TextSize::Small variant=TypographyVariant::Muted>
                    "Урок"
                </Text>
                <div class="mt-2">
                    <Dropdown
                        _options=lesson_items
                        _selected=selected_lesson
                        _placeholder=Signal::derive(|| "Выберите урок".to_string())
                        test_id=Signal::derive(move || format!("{}-lesson-dropdown", app_id_for_dropdown.clone()))
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
