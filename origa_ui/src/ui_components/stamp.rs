use leptos::prelude::*;

#[component]
pub fn Stamp(#[prop(optional, into)] text: Signal<String>) -> impl IntoView {
    view! {
        <div class="stamp">{move || text.get()}</div>
    }
}
