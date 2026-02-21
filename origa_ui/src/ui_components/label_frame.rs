use leptos::prelude::*;

#[component]
pub fn LabelFrame(
    #[prop(optional, into)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <div class=move || format!("label-frame {}", class.get())>
            {children()}
        </div>
    }
}
