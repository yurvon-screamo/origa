use leptos::prelude::*;

#[component]
pub fn HistoryButton(on_click: Callback<()>) -> impl IntoView {
    view! {
        <button
            class="cursor-pointer transition-colors duration-200 hover:opacity-70"
            on:click=move |_| on_click.run(())
        >
            <svg
                xmlns="http://www.w3.org/2000/svg"
                viewBox="0 0 24 24"
                class="w-4 h-4"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
            >
                <circle cx="12" cy="12" r="10" />
                <polyline points="12 6 12 12 16 14" />
            </svg>
        </button>
    }
}
