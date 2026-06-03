use leptos::prelude::*;

#[component]
pub fn WordTranslations(
    #[prop(into)] translations: Signal<Vec<String>>,
    #[prop(optional, into)] description: Signal<Option<String>>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    view! {
        <div class="word-translations" data-testid=move || test_id.get()>
            <ul class="word-translations-list">
                {move || {
                    translations
                        .get()
                        .into_iter()
                        .map(|t| view! { <li class="word-translations-item">{t}</li> })
                        .collect::<Vec<_>>()
                }}
            </ul>
            <Show when=move || description.get().is_some_and(|d| !d.is_empty())>
                <div class="word-translations-desc">
                    {move || description.get().unwrap_or_default()}
                </div>
            </Show>
        </div>
    }
}
