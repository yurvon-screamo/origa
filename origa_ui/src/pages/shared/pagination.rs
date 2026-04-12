use crate::i18n::use_i18n;
use leptos::prelude::*;

const DEFAULT_PAGE_SIZE: usize = 50;

#[component]
pub fn LoadMoreButton(
    visible_count: RwSignal<usize>,
    #[prop(into)] total: Signal<usize>,
    #[prop(default = DEFAULT_PAGE_SIZE)] page_size: usize,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let i18n = use_i18n();

    view! {
        <Show when=move || visible_count.get() < total.get()>
            <div class="flex justify-center mt-4">
                <button
                    class="px-6 py-2 text-sm font-medium transition-colors duration-200 border cursor-pointer border-[var(--border-dark)] bg-[var(--bg-paper)] hover:bg-[var(--bg-aged)]"
                    data-testid=move || test_id.get()
                    on:click=move |_| {
                        let t = total.get();
                        visible_count.update(|n| *n = (*n + page_size).min(t));
                    }
                >
                    {move || {
                        let remaining = total.get().saturating_sub(visible_count.get());
                        i18n.get_keys().shared().load_more().inner().to_string().replacen("{}", &remaining.to_string(), 1)
                    }}
                </button>
            </div>
        </Show>
    }
}
