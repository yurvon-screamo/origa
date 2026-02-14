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
    #[prop(optional)] variant: TagVariant,
    #[prop(optional, into)] class: String,
    children: Children,
) -> impl IntoView {
    let base_class = "tag";
    let variant_class = match variant {
        TagVariant::Default => "",
        TagVariant::Filled => "tag-filled",
        TagVariant::Olive => "tag-olive",
        TagVariant::Terracotta => "tag-terracotta",
    };

    let full_class = format!("{} {} {}", base_class, variant_class, class);

    view! {
        <span class=full_class>
            {children()}
        </span>
    }
}
