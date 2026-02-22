use leptos::prelude::*;

#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum TagVariant {
    #[default]
    Default,
    Filled,
    Olive,
    Terracotta,
}

#[component]
pub fn Tag(
    #[prop(optional, into)] variant: Signal<TagVariant>,
    #[prop(optional, into)] class: Signal<String>,
    #[prop(optional)] on_click: Option<Callback<leptos::ev::MouseEvent>>,
    children: Children,
) -> impl IntoView {
    let variant_class = move || match variant.get() {
        TagVariant::Default => "",
        TagVariant::Filled => "tag-filled",
        TagVariant::Olive => "tag-olive",
        TagVariant::Terracotta => "tag-terracotta",
    };

    let full_class = move || format!("tag {} {}", variant_class(), class.get());

    match on_click {
        Some(callback) => view! {
            <button
                class=full_class
                on:click=move |ev| callback.run(ev)
            >
                {children()}
            </button>
        }
        .into_any(),
        None => view! {
            <span class=full_class>
                {children()}
            </span>
        }
        .into_any(),
    }
}
