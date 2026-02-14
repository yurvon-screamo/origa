use leptos::prelude::*;

#[component]
pub fn Search(
    #[prop(optional)] value: RwSignal<String>,
    #[prop(optional, into)] placeholder: String,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    let full_class = format!("search-container {}", class);

    view! {
        <div class=full_class>
            <svg class="search-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                <circle cx="11" cy="11" r="8" />
                <path d="M21 21l-4.35-4.35" />
            </svg>
            <input
                type="text"
                class="search-input"
                placeholder=placeholder
                bind:value=value
            />
        </div>
    }
}
