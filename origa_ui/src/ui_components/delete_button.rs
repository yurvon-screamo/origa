use leptos::prelude::*;

#[component]
pub fn DeleteButton(on_click: Callback<()>) -> impl IntoView {
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
                <path d="M3 6h18M19 6v14a2 2 0 01-2 2H7a2 2 0 01-2-2V6m3 0V4a2 2 0 012-2h4a2 2 0 012 2v2" />
            </svg>
        </button>
    }
}
