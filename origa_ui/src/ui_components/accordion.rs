use leptos::prelude::*;

#[component]
pub fn AccordionItem(
    #[prop(optional, into)] header: Signal<String>,
    children: Children,
) -> impl IntoView {
    let (is_open, set_is_open) = signal(false);

    let toggle = move |_| {
        set_is_open.update(|open| *open = !*open);
    };

    view! {
        <div class=move || format!("accordion-item {}", if is_open.get() { "active" } else { "" })>
            <div
                class="accordion-header"
                on:click=toggle
            >
                <span class="font-mono text-xs tracking-wider">{move || header.get()}</span>
                <span class="accordion-icon"></span>
            </div>
            <div
                class="accordion-content"
                style:max-height=move || if is_open.get() { "200px" } else { "0px" }
            >
                <div class="accordion-body font-mono text-xs text-[var(--fg-muted)]">
                    {children()}
                </div>
            </div>
        </div>
    }
}
