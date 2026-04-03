use crate::ui_components::{Button, ButtonSize, ButtonVariant};
use leptos::ev::MouseEvent;
use leptos::prelude::*;

#[component]
pub fn CollapsibleDescription(
    #[prop(optional, default = true)] default_collapsed: bool,
    #[prop(optional, into)] test_id: Signal<String>,
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

    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    let button_test_id = Signal::derive(move || {
        let val = test_id.get();
        if val.is_empty() {
            String::new()
        } else {
            format!("{}-toggle", val)
        }
    });

    view! {
        <div data-testid=test_id_val>
            <div
                node_ref=content_ref
                class=move || if is_expanded.get() { "" } else { "line-clamp-3" }
            >
                {children()}
            </div>
            <Show when=move || needs_collapse.get()>
                <div class="collapsible-toggle">
                    <Button
                        variant=ButtonVariant::Ghost
                        size=ButtonSize::Small
                        test_id=button_test_id
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
