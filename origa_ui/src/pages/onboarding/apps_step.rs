use std::collections::HashSet;

use crate::ui_components::{Card, Checkbox, Text, TextSize, TypographyVariant};
use leptos::prelude::*;
use origa::traits::WellKnownSetMeta;

use super::onboarding_state::OnboardingState;

#[derive(Clone, Debug, PartialEq)]
struct AppInfo {
    id: String,
    name: String,
    description: String,
    icon: &'static str,
}

fn get_known_apps() -> Vec<AppInfo> {
    vec![
        AppInfo {
            id: "Migii".to_string(),
            name: "Migii".to_string(),
            description: "Приложение для изучения JLPT".to_string(),
            icon: "/public/external_icons/migii.png",
        },
        AppInfo {
            id: "DuolingoRu".to_string(),
            name: "Duolingo (RU)".to_string(),
            description: "Duolingo на русском языке".to_string(),
            icon: "/public/external_icons/duolingo.png",
        },
        AppInfo {
            id: "DuolingoEn".to_string(),
            name: "Duolingo (EN)".to_string(),
            description: "Duolingo на английском языке".to_string(),
            icon: "/public/external_icons/duolingo.png",
        },
        AppInfo {
            id: "MinnaNoNihongoN5".to_string(),
            name: "Minna no Nihongo N5".to_string(),
            description: "Учебник японского языка (уроки 1-25)".to_string(),
            icon: "/public/external_icons/minnanonihongo.png",
        },
        AppInfo {
            id: "MinnaNoNihongoN4".to_string(),
            name: "Minna no Nihongo N4".to_string(),
            description: "Учебник японского языка (уроки 26-50)".to_string(),
            icon: "/public/external_icons/minnanonihongo.png",
        },
    ]
}

fn get_available_app_ids(available_sets: &[WellKnownSetMeta]) -> HashSet<String> {
    let mut set_types: HashSet<String> = HashSet::new();
    let mut has_minna_n5 = false;
    let mut has_minna_n4 = false;

    for meta in available_sets {
        set_types.insert(meta.set_type.clone());
        if meta.set_type.contains("MinnaNoNihongo") {
            if meta.id.starts_with("minna_n5_") {
                has_minna_n5 = true;
            }
            if meta.id.starts_with("minna_n4_") {
                has_minna_n4 = true;
            }
        }
    }

    let mut result: HashSet<String> = HashSet::new();

    for app in get_known_apps() {
        if set_types.contains(&app.id)
            || (app.id == "MinnaNoNihongoN5" && has_minna_n5)
            || (app.id == "MinnaNoNihongoN4" && has_minna_n4)
        {
            result.insert(app.id);
        }
    }

    result
}

#[component]
pub fn AppsStep(#[prop(optional, into)] test_id: Signal<String>) -> impl IntoView {
    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() {
            None
        } else {
            Some(val)
        }
    };

    let state =
        use_context::<RwSignal<OnboardingState>>().expect("OnboardingState context not found");

    let known_apps = get_known_apps();

    let filtered_apps = Memo::new(move |_| {
        let available_sets = state.get().available_sets.clone();
        let available_ids = get_available_app_ids(&available_sets);
        known_apps
            .iter()
            .filter(|app| available_ids.contains(&app.id))
            .cloned()
            .collect::<Vec<_>>()
    });

    let toggle_app = Callback::new(move |app_id: String| {
        state.update(|s| {
            if s.selected_apps.contains(&app_id) {
                s.remove_app(&app_id);
            } else {
                s.add_app(&app_id);
            }
        });
    });

    view! {
        <div class="apps-step" data-testid=test_id_val>
            <div class="text-center mb-6">
                <Text size=TextSize::Large variant=TypographyVariant::Primary test_id=Signal::derive(|| "apps-step-title".to_string())>
                    "Какие приложения вы используете?"
                </Text>
                <div class="mt-2">
                    <Text size=TextSize::Small variant=TypographyVariant::Muted test_id=Signal::derive(|| "apps-step-subtitle".to_string())>
                        "Выберите приложения, в которых вы уже изучали японский. Мы импортируем ваш прогресс."
                    </Text>
                </div>
            </div>

            <div class="grid gap-4">
                    <For
                        each=move || filtered_apps.get()
                        key=|app| app.id.clone()
                        children=move |app| {
                            let app_id = app.id.clone();
                            let app_id_for_click = app_id.clone();
                            let app_id_for_cb = app_id.clone();
                            let app_id_for_selected = app_id.clone();
                            let app_name = app.name.clone();
                            let app_desc = app.description.clone();
                            let app_icon = app.icon;
                            let is_selected = Memo::new(move |_| state.get().selected_apps.contains(&app_id_for_selected));

                            let app_test_id = format!("apps-step-app-{}", app_id);
                            let app_test_id_for_card = app_test_id.clone();
                            let app_test_id_for_name = app_test_id.clone();
                            let app_test_id_for_desc = app_test_id.clone();
                            let app_test_id_for_checkbox = app_test_id.clone();

                            view! {
                                <Card
                                    class=Signal::derive(move || {
                                        let base = "card card-shadow card-selectable";
                                        if is_selected.get() {
                                            format!("{} selected", base)
                                        } else {
                                            base.to_string()
                                        }
                                    })
                                    test_id=Signal::derive(move || app_test_id_for_card.clone())
                                >
                                    <div
                                        class="flex items-center gap-4 p-2"
                                        on:click=move |_| {
                                            toggle_app.run(app_id_for_click.clone());
                                        }
                                    >
                                        <img src=app_icon class="w-12 h-12 object-contain" alt=app_name.clone() />
                                        <div class="flex-1">
                                            <Text size=TextSize::Default variant=TypographyVariant::Primary test_id=Signal::derive(move || format!("{}-name", app_test_id_for_name.clone()))>
                                                {app_name}
                                            </Text>
                                            <Text size=TextSize::Small variant=TypographyVariant::Muted test_id=Signal::derive(move || format!("{}-desc", app_test_id_for_desc.clone()))>
                                                {app_desc}
                                            </Text>
                                        </div>
                                        <Checkbox
                                            checked=Signal::derive(move || is_selected.get())
                                            label=Signal::derive(String::new)
                                            on_change=Callback::new(move |()| {
                                                toggle_app.run(app_id_for_cb.clone());
                                            })
                                            test_id=Signal::derive(move || format!("{}-checkbox", app_test_id_for_checkbox.clone()))
                                        />
                                    </div>
                                </Card>
                            }
                        }
                    />
            </div>

            <Show when=move || state.get().selected_apps.is_empty()>
                <div class="text-center mt-4">
                    <Text size=TextSize::Small variant=TypographyVariant::Muted test_id=Signal::derive(|| "apps-step-skip-hint".to_string())>
                        "Можно пропустить этот шаг, если вы не использовали другие приложения"
                    </Text>
                </div>
            </Show>
        </div>
    }
}
