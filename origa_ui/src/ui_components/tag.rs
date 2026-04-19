use leptos::prelude::*;

#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum TagVariant {
    #[default]
    Default,
    Filled,
    Olive,
    Terracotta,
    Sage,
}

#[component]
pub fn Tag(
    #[prop(optional, into)] variant: Signal<TagVariant>,
    #[prop(optional, into)] class: Signal<String>,
    #[prop(optional)] on_click: Option<Callback<leptos::ev::MouseEvent>>,
    #[prop(optional, into)] test_id: Signal<String>,
    children: Children,
) -> impl IntoView {
    let variant_class = move || match variant.get() {
        TagVariant::Default => "",
        TagVariant::Filled => "tag-filled",
        TagVariant::Olive => "tag-olive",
        TagVariant::Terracotta => "tag-terracotta",
        TagVariant::Sage => "tag-sage",
    };

    let is_clickable = on_click.is_some();
    let anima_class = if is_clickable { "anima-tag-hover" } else { "" };
    let full_class = move || format!("tag {} {} {}", variant_class(), anima_class, class.get());

    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    match on_click {
        Some(callback) => view! {
            <button
                class=full_class
                on:click=move |ev| callback.run(ev)
                data-testid=test_id_val
            >
                {children()}
            </button>
        }
        .into_any(),
        None => view! {
            <span class=full_class data-testid=test_id_val>
                {children()}
            </span>
        }
        .into_any(),
    }
}
