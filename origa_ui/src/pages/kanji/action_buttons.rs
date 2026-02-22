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
        <div class="flex justify-end gap-4">
            <Button
                variant=ButtonVariant::Default
                on_click=on_cancel
                disabled=is_creating
            >
                "Отмена"
            </Button>
            <Button
                variant=ButtonVariant::Olive
                on_click=on_add
                disabled=Signal::derive(move || is_disabled.get() || is_creating.get())
            >
                {move || if is_creating.get() { "Добавление..." } else { "Добавить" }}
            </Button>
        </div>
    }
}
