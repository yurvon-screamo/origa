use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use leptos_use::use_event_listener;

#[derive(Clone, Debug)]
pub struct DropdownItem {
    pub value: String,
    pub label: String,
}

#[component]
pub fn Dropdown(
    #[prop(into)] options: Vec<DropdownItem>,
    #[prop(optional)] selected: RwSignal<String>,
    #[prop(optional, into)] placeholder: String,
) -> impl IntoView {
    let (is_open, set_is_open) = signal(false);
    let dropdown_ref = NodeRef::<leptos::html::Div>::new();

    let toggle_dropdown = move |_| {
        set_is_open.update(|open| *open = !*open);
    };

    let select_item = move |item: DropdownItem| {
        selected.set(item.value.clone());
        set_is_open.set(false);
    };

    let close_on_outside = move |ev: leptos::ev::MouseEvent| {
        let target = ev.target();
        let mut should_close = true;

        if let Some(el) = dropdown_ref.get()
            && let Some(target) = target
        {
            let target_node: Option<web_sys::Node> = target.dyn_into().ok();
            let el_node: &web_sys::Node = &el;
            should_close = !el_node.contains(target_node.as_ref());
        }

        if should_close {
            set_is_open.set(false);
        }
    };

    let _ = use_event_listener(document(), leptos::ev::click, close_on_outside);

    let display_text = {
        let options = options.clone();
        move || {
            let sel = selected.get();
            options
                .iter()
                .find(|opt| opt.value == sel)
                .map(|opt| opt.label.clone())
                .unwrap_or_else(|| placeholder.clone())
        }
    };

    view! {
        <div
            class=format!("dropdown {}", if is_open.get() { "active" } else { "" })
            node_ref=dropdown_ref
        >
            <button
                class="dropdown-trigger"
                type="button"
                on:click=toggle_dropdown
            >
                {display_text()}
            </button>
            <div class="dropdown-menu">
                <For
                    each=move || options.clone()
                    key=|item| item.value.clone()
                    children=move |item| {
                        let item_clone = item.clone();
                        view! {
                            <div
                                class="dropdown-item"
                                on:click=move |_| select_item(item_clone.clone())
                            >
                                {item.label}
                            </div>
                        }
                    }
                />
            </div>
        </div>
    }
}
