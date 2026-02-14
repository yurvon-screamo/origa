use leptos::prelude::*;

#[component]
pub fn LabelFrame(#[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    let full_class = format!("label-frame {}", class);

    view! {
        <div class=full_class>
            {children()}
        </div>
    }
}
