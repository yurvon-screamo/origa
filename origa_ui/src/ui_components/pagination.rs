use leptos::prelude::*;

#[component]
pub fn Pagination(
    #[prop(optional)] current_page: RwSignal<usize>,
    #[prop(optional, into)] total_pages: Signal<usize>,
) -> impl IntoView {
    let handle_page_change = move |page: usize| {
        current_page.set(page);
    };

    let pages = move || {
        let current = current_page.get();
        let total = total_pages.get();
        let mut result = vec![];

        if current > 1 {
            result.push((0, "←".to_string()));
        }

        for i in 1..=total.min(5) {
            result.push((i, i.to_string()));
        }

        if total > 5 {
            result.push((0, "...".to_string()));
            result.push((total, total.to_string()));
        }

        if current < total {
            result.push((total + 1, "→".to_string()));
        }

        result
    };

    view! {
        <div class="pagination">
            <For
                each=move || pages()
                key=|(idx, label)| format!("{}-{}", idx, label)
                children=move |(idx, label)| {
                    let is_current = move || current_page.get() == idx;
                    let is_disabled = move || idx == 0 || idx > total_pages.get();
                    view! {
                        <button
                            class=move || format!("pagination-btn {}", if is_current() { "active" } else { "" })
                            disabled=move || is_disabled()
                            on:click=move |_| {
                                if idx > 0 && idx <= total_pages.get() {
                                    handle_page_change(idx);
                                }
                            }
                        >
                            {label}
                        </button>
                    }
                }
            />
        </div>
    }
}
