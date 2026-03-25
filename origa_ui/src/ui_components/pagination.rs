use leptos::prelude::*;

#[component]
pub fn Pagination(
    #[prop(optional)] _current_page: RwSignal<usize>,
    #[prop(optional, into)] _total_pages: Signal<usize>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    let button_test_id = move |idx: usize, total: usize| {
        let base = test_id.get();
        if base.is_empty() {
            return None;
        }
        if idx == 0 {
            return Some(format!("{}-prev", base));
        }
        if idx > total {
            return Some(format!("{}-next", base));
        }
        Some(format!("{}-page-{}", base, idx))
    };

    let handle_page_change = move |page: usize| {
        _current_page.set(page);
    };

    let pages = move || {
        let current = _current_page.get();
        let total = _total_pages.get();
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
        <div class="pagination" data-testid=test_id_val>
            <For
                each=move || pages()
                key=|(idx, label)| format!("{}-{}", idx, label)
                children=move |(idx, label)| {
                    let is_current = move || _current_page.get() == idx;
                    let is_disabled = move || idx == 0 || idx > _total_pages.get();
                    let btn_test_id = move || button_test_id(idx, _total_pages.get());
                    view! {
                        <button
                            class=move || format!("pagination-btn {}", if is_current() { "active" } else { "" })
                            disabled=is_disabled
                            data-testid=btn_test_id
                            on:click=move |_| {
                                if idx > 0 && idx <= _total_pages.get() {
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
