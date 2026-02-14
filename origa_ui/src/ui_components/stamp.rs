use leptos::prelude::*;

#[component]
pub fn Stamp(#[prop(optional, into)] text: String) -> impl IntoView {
    view! {
        <div class="stamp">{text}</div>
    }
}
