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
    children: Children,
) -> impl IntoView {
    view! {
        <span class=move || {
            let variant_class = match variant.get() {
                TagVariant::Default => "",
                TagVariant::Filled => "tag-filled",
                TagVariant::Olive => "tag-olive",
                TagVariant::Terracotta => "tag-terracotta",
            };
            format!("tag {} {}", variant_class, class.get())
        }>
            {children()}
        </span>
    }
}
