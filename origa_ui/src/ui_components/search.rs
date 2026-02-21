use leptos::prelude::*;

#[component]
pub fn Search(
    #[prop(optional)] value: RwSignal<String>,
    #[prop(optional, into)] placeholder: Signal<String>,
    #[prop(optional, into)] class: Signal<String>,
) -> impl IntoView {
    view! {
        <div class=move || format!("search-container {}", class.get())>
            <svg class="search-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                <circle cx="11" cy="11" r="8" />
                <path d="M21 21l-4.35-4.35" />
            </svg>
            <input
                type="text"
                class="search-input"
                placeholder=move || placeholder.get()
                bind:value=value
            />
        </div>
    }
}
