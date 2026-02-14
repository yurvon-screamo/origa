use leptos::prelude::*;

#[component]
pub fn LoginHeader() -> impl IntoView {
    view! {
        <div class="text-center mb-12">
            <h1 class="font-serif text-4xl font-light tracking-tight mb-4">
                "Origa"
            </h1>
            <p class="font-mono text-sm text-[var(--fg-muted)]">
                "Изучение японского языка"
            </p>
        </div>
    }
}
