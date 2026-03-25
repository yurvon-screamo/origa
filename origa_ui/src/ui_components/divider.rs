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
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() {
            None
        } else {
            Some(val)
        }
    };

    view! {
        <div
            class=move || {
                let divider_class = match variant.get() {
                    DividerVariant::Single => "divider",
                    DividerVariant::Double => "divider-double",
                };
                format!("{} {}", divider_class, class.get())
            }
            data-testid=test_id_val
        ></div>
    }
}
