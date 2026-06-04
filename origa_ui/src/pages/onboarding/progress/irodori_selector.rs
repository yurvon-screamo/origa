use crate::core::config::public_url;
use crate::i18n::{t, use_i18n};
use crate::ui_components::{Card, Dropdown, Text, TextSize, TypographyVariant};
use leptos::prelude::*;

use super::super::onboarding_state::OnboardingState;
use super::irodori_helpers::{
    build_book_items, build_lesson_items, collect_lessons_to_import, is_irodori_lesson,
};
use super::types::IrodoriLesson;

#[component]
pub fn IrodoriProgressSelector(
    lessons_nyuumon: Signal<Vec<IrodoriLesson>>,
    lessons_shokyuu1: Signal<Vec<IrodoriLesson>>,
    lessons_shokyuu2: Signal<Vec<IrodoriLesson>>,
    state: RwSignal<OnboardingState>,
) -> impl IntoView {
    let i18n = use_i18n();
    let selected_book = RwSignal::new("none".to_string());
    let selected_lesson = RwSignal::new("none".to_string());
    let available_sets = Signal::derive(move || state.get().available_sets.clone());

    let book_items = build_book_items(&i18n);

    let lesson_items = Signal::derive(move || {
        build_lesson_items(
            &i18n,
            &lessons_nyuumon.get(),
            &lessons_shokyuu1.get(),
            &lessons_shokyuu2.get(),
            &selected_book.get(),
        )
    });

    let import_info = Signal::derive(move || {
        let book = selected_book.get();
        let lesson = selected_lesson
            .get()
            .strip_prefix("lesson_")
            .and_then(|s| s.parse::<usize>().ok());

        match (book.as_str(), lesson) {
            ("nyuumon", Some(n)) => Some(
                i18n.get_keys()
                    .onboarding()
                    .progress()
                    .import_irodori_nyuumon()
                    .inner()
                    .to_string()
                    .replacen("{}", &n.to_string(), 1),
            ),
            ("shokyuu1", Some(n)) => Some(
                i18n.get_keys()
                    .onboarding()
                    .progress()
                    .import_irodori_shokyuu1()
                    .inner()
                    .to_string()
                    .replacen("{}", &n.to_string(), 1),
            ),
            ("shokyuu2", Some(n)) => Some(
                i18n.get_keys()
                    .onboarding()
                    .progress()
                    .import_irodori_shokyuu2()
                    .inner()
                    .to_string()
                    .replacen("{}", &n.to_string(), 1),
            ),
            _ => None,
        }
    });

    Effect::new(move |_| {
        let book = selected_book.get();
        let lesson_num = selected_lesson
            .get()
            .strip_prefix("lesson_")
            .and_then(|s| s.parse::<usize>().ok());

        if book == "none" || lesson_num.is_none() {
            return;
        }

        let nyuumon_snapshot: Vec<_> = lessons_nyuumon.get_untracked();
        let shokyuu1_snapshot: Vec<_> = lessons_shokyuu1.get_untracked();
        let shokyuu2_snapshot: Vec<_> = lessons_shokyuu2.get_untracked();
        let sets_snapshot: Vec<_> = available_sets.get_untracked();

        if let Some(n) = lesson_num {
            let ids_to_import = collect_lessons_to_import(
                &nyuumon_snapshot,
                &shokyuu1_snapshot,
                &shokyuu2_snapshot,
                &book,
                n,
            );

            state.update(|s| {
                s.set_app_selection("Irodori", &format!("{}_{}", book, n));
                s.sets_to_import.retain(|set| !is_irodori_lesson(&set.id));
                let sets_to_add: Vec<_> = sets_snapshot
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

    view! {
        <Card class=Signal::derive(|| "p-4".to_string())>
            <div class="flex items-center gap-3 mb-2">
                <img
                    src=public_url("/public/external_icons/irodori.png")
                    class="w-12 h-12 object-contain"
                    alt="Irodori"
                />
                <Text size=TextSize::Default variant=TypographyVariant::Primary>
                    "Irodori"
                </Text>
            </div>

            <div class="mt-4 space-y-4">
                <div>
                    <Text size=TextSize::Small variant=TypographyVariant::Muted>
                        {t!(i18n, onboarding.progress.book)}
                    </Text>
                    <div class="mt-2">
                        <Dropdown
                            options=Signal::derive(move || book_items.clone())
                            selected=selected_book
                            placeholder=Signal::derive(move || i18n.get_keys().onboarding().progress().select_book().inner().to_string())
                            test_id=Signal::derive(|| "irodori-book-dropdown".to_string())
                        />
                    </div>
                </div>

                <Show when=move || selected_book.get() != "none">
                    <div>
                        <Text size=TextSize::Small variant=TypographyVariant::Muted>
                            {t!(i18n, onboarding.progress.lesson)}
                        </Text>
                        <div class="mt-2">
                            <Dropdown
                                options=lesson_items
                                selected=selected_lesson
                                placeholder=Signal::derive(move || i18n.get_keys().onboarding().progress().select_lesson().inner().to_string())
                                test_id=Signal::derive(|| "irodori-lesson-dropdown".to_string())
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
