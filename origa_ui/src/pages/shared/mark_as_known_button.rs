use leptos::prelude::*;

#[component]
pub fn MarkAsKnownButton(
    on_click: Callback<()>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    view! {
        <button
            class="cursor-pointer transition-colors duration-200 hover:opacity-70"
            on:click=move |_| on_click.run(())
            data-testid=test_id_val
            title="Отметить как изученное"
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
                <path d="M20 6 9 17l-5-5" />
            </svg>
        </button>
    }
}
