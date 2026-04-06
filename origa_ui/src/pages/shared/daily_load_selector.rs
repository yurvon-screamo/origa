use crate::pages::icons::INFO_ICON;
use crate::ui_components::{Button, ButtonVariant, Tooltip};
use leptos::prelude::*;
use origa::domain::DailyLoad;

#[component]
pub fn DailyLoadSelector(selected_load: RwSignal<DailyLoad>) -> impl IntoView {
    view! {
        <div class="flex flex-wrap justify-start gap-2 mt-2">
            <For
                each=move || DailyLoad::all().to_vec()
                key=|load| format!("{:?}", load)
                children=move |load| {
                    let is_selected = move || selected_load.get() == load;
                    let description = load.description().to_string();
                    let load_id = format!("{:?}", load).to_lowercase();
                    view! {
                        <Tooltip text=Signal::derive(move || description.clone())>
                            <Button
                                variant=move || if is_selected() { ButtonVariant::Olive } else { ButtonVariant::Default }
                                on_click={Callback::new(move |_| selected_load.set(load))}
                                test_id=Signal::derive(move || format!("profile-load-{}", load_id))
                            >
                                <span class="inline-flex items-center">
                                    {load.as_str().to_string()}
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
