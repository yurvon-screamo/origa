use leptos::prelude::*;

#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum DividerVariant {
    #[default]
    Single,
    Double,
}

#[component]
pub fn Divider(
    #[prop(optional)] variant: DividerVariant,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    let divider_class = match variant {
        DividerVariant::Single => "divider",
        DividerVariant::Double => "divider-double",
    };

    let full_class = format!("{} {}", divider_class, class);

    view! {
        <div class=full_class></div>
    }
}
