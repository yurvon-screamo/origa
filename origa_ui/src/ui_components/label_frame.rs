use leptos::prelude::*;

#[component]
pub fn LabelFrame(
    #[prop(optional, into)] _class: Signal<String>,
    _children: Children,
) -> impl IntoView {
    view! {
        <div class=move || format!("label-frame {}", _class.get())>
            {_children()}
        </div>
    }
}
