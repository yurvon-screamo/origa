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
    #[prop(optional, into)] _options: Signal<Vec<DropdownItem>>,
    _selected: RwSignal<String>,
    #[prop(optional, into)] _placeholder: Signal<String>,
) -> impl IntoView {
    let is_open = RwSignal::new(false);
    let search_query = RwSignal::new(String::new());
    let dropdown_ref = NodeRef::<leptos::html::Div>::new();
    let input_ref = NodeRef::<leptos::html::Input>::new();

    Effect::new(move |_| {
        if is_open.get()
            && let Some(input) = input_ref.get()
        {
            let _ = input.focus();
        }
    });

    let filtered_options = Signal::derive(move || {
        let query = search_query.get();
        let lower_query = query.to_lowercase();
        _options
            .get()
            .into_iter()
            .filter(|item| {
                if lower_query.is_empty() {
                    return true;
                }
                item.label.to_lowercase().contains(&lower_query)
            })
            .collect::<Vec<_>>()
    });

    let toggle_dropdown = move |_| {
        is_open.update(|open| {
            *open = !*open;
            if !*open {
                search_query.set(String::new());
            }
        });
    };

    let select_item = move |item: DropdownItem| {
        _selected.set(item.value.clone());
        is_open.set(false);
        search_query.set(String::new());
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
            is_open.set(false);
            search_query.set(String::new());
        }
    };

    let _ = use_event_listener(document(), leptos::ev::click, close_on_outside);

    let display_text = move || {
        let sel = _selected.get();
        _options
            .get()
            .iter()
            .find(|opt| opt.value == sel)
            .map(|opt| opt.label.clone())
            .unwrap_or_else(|| _placeholder.get())
    };

    let on_search_input = move |ev: leptos::ev::Event| {
        if let Some(input) = ev.target().and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok()) {
            search_query.set(input.value());
        }
    };

    Effect::new(move |_| {
        if let Some(input) = input_ref.get() {
            input.set_value(&search_query.get());
        }
    });

    view! {
        <div
            class=move || format!("dropdown {}", if is_open.get() { "active" } else { "" })
            node_ref=dropdown_ref
        >
            <button
                class="dropdown-trigger"
                type="button"
                on:click=toggle_dropdown
            >
                {display_text}
            </button>
            <div class="dropdown-menu">
                <div class="dropdown-search">
                    <input
                        type="text"
                        placeholder="Поиск..."
                        node_ref=input_ref
                        on:input=on_search_input
                    />
                </div>
                <div class="dropdown-items">
                    <For
                        each=move || filtered_options.get()
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
        </div>
    }
}