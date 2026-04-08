use crate::i18n::*;
use crate::pages::icons::INFO_ICON;
use crate::ui_components::{Button, ButtonVariant, Tooltip};
use leptos::prelude::*;
use origa::domain::DailyLoad;

#[component]
pub fn DailyLoadSelector(selected_load: RwSignal<DailyLoad>) -> impl IntoView {
    let i18n = use_i18n();

    view! {
        <div class="flex flex-wrap justify-start gap-2 mt-2">
            <For
                each=move || DailyLoad::all().to_vec()
                key=|load| format!("{:?}", load)
                children=move |load| {
                    let is_selected = move || selected_load.get() == load;
                    let load_for_click = load;
                    let load_id = format!("{:?}", load).to_lowercase();
                    let label = match load {
                        DailyLoad::Light => i18n.get_keys().shared().load_light().inner().to_string(),
                        DailyLoad::Medium => i18n.get_keys().shared().load_medium().inner().to_string(),
                        DailyLoad::Hard => i18n.get_keys().shared().load_hard().inner().to_string(),
                        DailyLoad::Impossible => i18n.get_keys().shared().load_impossible().inner().to_string(),
                    };
                    let description = match load {
                        DailyLoad::Light => i18n.get_keys().shared().load_light_desc().inner().to_string(),
                        DailyLoad::Medium => i18n.get_keys().shared().load_medium_desc().inner().to_string(),
                        DailyLoad::Hard => i18n.get_keys().shared().load_hard_desc().inner().to_string(),
                        DailyLoad::Impossible => i18n.get_keys().shared().load_impossible_desc().inner().to_string(),
                    };

                    view! {
                        <Tooltip text=Signal::derive(move || description.clone())>
                            <Button
                                variant=move || if is_selected() { ButtonVariant::Olive } else { ButtonVariant::Default }
                                on_click=Callback::new(move |_| selected_load.set(load_for_click))
                                test_id=Signal::derive(move || format!("profile-load-{}", load_id))
                            >
                                <span class="inline-flex items-center">
                                    {label}
                                    <span inner_html=INFO_ICON />
                                </span>
                            </Button>
                        </Tooltip>
                    }
                }
            />
        </div>
    }
}
