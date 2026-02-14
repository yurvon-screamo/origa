use leptos::prelude::*;

#[component]
pub fn Tooltip(#[prop(optional, into)] text: String, children: Children) -> impl IntoView {
    view! {
        <div class="tooltip-container">
            {children()}
            <div class="tooltip">{text}</div>
        </div>
    }
}
