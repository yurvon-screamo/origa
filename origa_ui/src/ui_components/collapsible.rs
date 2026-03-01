use crate::ui_components::{Button, ButtonVariant};
use leptos::ev::MouseEvent;
use leptos::prelude::*;

#[component]
pub fn CollapsibleDescription(
    #[prop(optional, default = true)] default_collapsed: bool,
    children: Children,
) -> impl IntoView {
    let is_expanded = RwSignal::new(!default_collapsed);
    let content_ref = NodeRef::<leptos::html::Div>::new();
    let needs_collapse = RwSignal::new(false);

    Effect::new(move |_| {
        if let Some(el) = content_ref.get() {
            let is_overflow = el.scroll_height() > el.client_height();
            needs_collapse.set(is_overflow);
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
            <Show when=move || needs_collapse.get()>
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
            </Show>
        </div>
    }
}
