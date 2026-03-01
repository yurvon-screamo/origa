use crate::ui_components::{Button, ButtonVariant};
use leptos::ev::MouseEvent;
use leptos::prelude::*;

#[component]
pub fn CollapsibleDescription(
    #[prop(optional, default = true)] default_collapsed: bool,
    children: Children,
) -> impl IntoView {
    let is_expanded = RwSignal::new(!default_collapsed);

    view! {
        <div>
            <div class=move || if is_expanded.get() { "" } else { "line-clamp-3" }>
                {children()}
            </div>
            <div class="mt-2">
                <Button
                    variant=ButtonVariant::Ghost
                    on_click=Callback::new(move |_: MouseEvent| {
                        is_expanded.update(|v| *v = !*v);
                    })
                >
                    {move || if is_expanded.get() { "Свернуть" } else { "Развернуть" }}
                </Button>
            </div>
        </div>
    }
}
