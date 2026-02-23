use leptos::prelude::*;

#[component]
pub fn ErrorMessage(#[prop(into)] message: String) -> impl IntoView {
    view! {
        <div class="p-3 bg-red-950/20 border border-red-900/30 text-red-400 font-mono text-xs">
            {message}
        </div>
    }
}
