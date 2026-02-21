use leptos::prelude::*;

#[component]
pub fn Tooltip(#[prop(optional, into)] text: Signal<String>, children: Children) -> impl IntoView {
    view! {
        <div class="tooltip-container">
            {children()}
            <div class="tooltip">{move || text.get()}</div>
        </div>
    }
}
