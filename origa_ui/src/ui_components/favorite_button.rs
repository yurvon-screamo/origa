use leptos::prelude::*;

#[component]
pub fn FavoriteButton(
    #[prop(into)] is_favorite: Signal<bool>,
    on_click: Callback<()>,
    #[prop(optional, into)] pending: Signal<bool>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    view! {
        <button
            class="icon-btn anima-press favorite-button"
            disabled=move || pending.get()
            on:click=move |ev: leptos::ev::MouseEvent| {
                ev.stop_propagation();
                on_click.run(());
            }
            data-testid=test_id_val
            aria-label="Toggle favorite"
        >
            <Show
                when=move || pending.get()
                fallback=move || view! {
                    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
                        <Show
                            when=move || is_favorite.get()
                            fallback=move || view! {
                                <path
                                    fill="none"
                                    stroke="currentColor"
                                    stroke-width="2"
                                    stroke-linecap="round"
                                    stroke-linejoin="round"
                                    d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z"
                                />
                            }
                        >
                            <path
                                fill="currentColor"
                                d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z"
                            />
                        </Show>
                    </svg>
                }
            >
                <span class="spinner spinner-sm" aria-hidden="true"></span>
            </Show>
        </button>
    }
}
