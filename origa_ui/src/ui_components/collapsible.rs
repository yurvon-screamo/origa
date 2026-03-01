use crate::ui_components::{Button, ButtonVariant};
use leptos::ev::MouseEvent;
use leptos::prelude::*;

#[component]
pub fn CollapsibleDescription(
    #[prop(optional, default = true)] default_collapsed: bool,
    children: Children,
) -> impl IntoView {
    let is_expanded = RwSignal::new(!default_collapsed);
    let show_button = RwSignal::new(false);
    let content_ref = NodeRef::new();

    Effect::new(move |_| {
        if let Some(el) = content_ref.get() {
            let is_overflowing = el.scroll_height() > el.client_height();
            show_button.set(is_overflowing);
        }
    });

    view! {
        <div>
            <div
                node_ref=content_ref
                class=move || if is_expanded.get() { "" } else { "line-clamp-3" }
            >
                {children()}
            </div>
            <div class=move || if show_button.get() { "mt-2" } else { "hidden" }>
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
