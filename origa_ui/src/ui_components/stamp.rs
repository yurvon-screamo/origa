use leptos::prelude::*;

#[component]
pub fn Stamp(#[prop(optional, into)] _text: Signal<String>) -> impl IntoView {
    view! {
        <div class="stamp">{move || _text.get()}</div>
    }
}
