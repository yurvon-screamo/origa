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
            icon: "📱",
        },
        AppInfo {
            id: "DuolingoRu".to_string(),
            name: "Duolingo (RU)".to_string(),
            description: "Duolingo на русском языке".to_string(),
            icon: "🦉",
        },
        AppInfo {
            id: "DuolingoEn".to_string(),
            name: "Duolingo (EN)".to_string(),
            description: "Duolingo на английском языке".to_string(),
            icon: "🦉",
        },
        AppInfo {
            id: "MinnaNoNihongo".to_string(),
            name: "Minna no Nihongo".to_string(),
            description: "Учебник японского языка".to_string(),
            icon: "📚",
        },
    ]
}

fn get_available_app_ids(available_sets: &[WellKnownSetMeta]) -> HashSet<String> {
    let mut set_types: HashSet<String> = HashSet::new();
    for meta in available_sets {
        set_types.insert(meta.set_type.clone());
    }
    get_known_apps()
        .into_iter()
        .filter(|app| set_types.contains(&app.id))
        .map(|app| app.id)
        .collect()
}

#[component]
pub fn AppsStep() -> impl IntoView {
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
        <div class="apps-step">
            <div class="text-center mb-6">
                <Text size=TextSize::Large variant=TypographyVariant::Primary>
                    "Какие приложения вы используете?"
                </Text>
                <div class="mt-2">
                    <Text size=TextSize::Small variant=TypographyVariant::Muted>
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

                        view! {
                            <Card
                                class=Signal::derive(move || {
                                    if is_selected.get() {
                                        "border-2 border-olive-500 bg-olive-50 cursor-pointer".to_string()
                                    } else {
                                        "border-2 border-transparent hover:border-gray-200 cursor-pointer".to_string()
                                    }
                                })
                            >
                                <div
                                    class="flex items-center gap-4 p-2"
                                    on:click=move |_| {
                                        toggle_app.run(app_id_for_click.clone());
                                    }
                                >
                                    <div class="text-3xl">{app_icon}</div>
                                    <div class="flex-1">
                                        <Text size=TextSize::Default variant=TypographyVariant::Primary>
                                            {app_name}
                                        </Text>
                                        <Text size=TextSize::Small variant=TypographyVariant::Muted>
                                            {app_desc}
                                        </Text>
                                    </div>
                                    <Checkbox
                                        checked=Signal::derive(move || is_selected.get())
                                        label=Signal::derive(|| String::new())
                                        on_change=Callback::new(move |()| {
                                            toggle_app.run(app_id_for_cb.clone());
                                        })
                                    />
                                </div>
                            </Card>
                        }
                    }
                />
            </div>

            <Show when=move || state.get().selected_apps.is_empty()>
                <div class="text-center mt-4">
                    <Text size=TextSize::Small variant=TypographyVariant::Muted>
                        "Можно пропустить этот шаг, если вы не использовали другие приложения"
                    </Text>
                </div>
            </Show>
        </div>
    }
}
