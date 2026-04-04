use crate::ui_components::{Button, ButtonVariant};
use leptos::prelude::*;
use origa::domain::DailyLoad;

#[component]
pub fn DailyLoadSelector(selected_load: RwSignal<DailyLoad>) -> impl IntoView {
    view! {
        <div class="flex flex-wrap gap-2 mt-2">
            <For
                each=move || DailyLoad::all().to_vec()
                key=|load| format!("{:?}", load)
                children=move |load| {
                    let is_selected = move || selected_load.get() == load;
                    view! {
                        <Button
                            variant=move || if is_selected() { ButtonVariant::Olive } else { ButtonVariant::Default }
                            on_click={Callback::new(move |_| selected_load.set(load))}
                        >
                            {load.as_str().to_string()}
                        </Button>
                    }
                }
            />
        </div>
    }
}
