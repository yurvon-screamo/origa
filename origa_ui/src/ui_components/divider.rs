use leptos::prelude::*;

#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum DividerVariant {
    #[default]
    Single,
    Double,
}

#[component]
pub fn Divider(
    #[prop(optional, into)] variant: Signal<DividerVariant>,
    #[prop(optional, into)] class: Signal<String>,
) -> impl IntoView {
    view! {
        <div class=move || {
            let divider_class = match variant.get() {
                DividerVariant::Single => "divider",
                DividerVariant::Double => "divider-double",
            };
            format!("{} {}", divider_class, class.get())
        }></div>
    }
}
