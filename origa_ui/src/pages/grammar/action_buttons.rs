use crate::ui_components::{Button, ButtonVariant};
use leptos::prelude::*;

#[component]
pub fn ActionButtons(
    is_creating: RwSignal<bool>,
    is_disabled: Signal<bool>,
    on_cancel: Callback<leptos::ev::MouseEvent>,
    on_add: Callback<leptos::ev::MouseEvent>,
) -> impl IntoView {
    view! {
        <div class="flex gap-2 justify-end">
            <Button variant=ButtonVariant::Ghost on_click=on_cancel>
                "Отмена"
            </Button>
            <Button
                variant=ButtonVariant::Olive
                disabled=Signal::derive(move || is_creating.get() || is_disabled.get())
                on_click=on_add
            >
                {move || if is_creating.get() { "Добавление..." } else { "Добавить" }}
            </Button>
        </div>
    }
}
