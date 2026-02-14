use leptos::prelude::*;

#[component]
pub fn Pagination(
    #[prop(optional)] current_page: RwSignal<usize>,
    #[prop(default = 1)] total_pages: usize,
) -> impl IntoView {
    let handle_page_change = move |page: usize| {
        current_page.set(page);
    };

    let pages = move || {
        let current = current_page.get();
        let mut result = vec![];

        if current > 1 {
            result.push((0, "←".to_string()));
        }

        for i in 1..=total_pages.min(5) {
            result.push((i, i.to_string()));
        }

        if total_pages > 5 {
            result.push((0, "...".to_string()));
            result.push((total_pages, total_pages.to_string()));
        }

        if current < total_pages {
            result.push((total_pages + 1, "→".to_string()));
        }

        result
    };

    view! {
        <div class="pagination">
            <For
                each=move || pages()
                key=|(idx, _)| *idx
                children=move |(idx, label)| {
                    let is_current = move || current_page.get() == idx;
                    let is_disabled = move || idx == 0 || idx > total_pages;
                    view! {
                        <button
                            class=format!("pagination-btn {}", if is_current() { "active" } else { "" })
                            disabled=is_disabled()
                            on:click=move |_| {
                                if idx > 0 && idx <= total_pages {
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
