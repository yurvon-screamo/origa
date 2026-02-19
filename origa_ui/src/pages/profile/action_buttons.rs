use crate::ui_components::{Button, ButtonVariant};
use leptos::ev::MouseEvent;
use leptos::prelude::*;

#[component]
pub fn ActionButtons(
    on_save: Callback<MouseEvent>,
    on_logout: Callback<MouseEvent>,
    is_saving: Signal<bool>,
) -> impl IntoView {
    view! {
        <div class="flex space-x-4">
            <Button
                variant={ButtonVariant::Filled}
                on_click={on_save}
                disabled=is_saving
            >
                {move || if is_saving.get() { "Сохранение..." } else { "Сохранить изменения" }}
            </Button>

            <Button
                variant={ButtonVariant::Ghost}
                on_click={on_logout}
            >
                "Выйти из аккаунта"
            </Button>
        </div>
    }
}
