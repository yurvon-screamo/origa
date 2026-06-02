use crate::ui_components::MarkdownText;
use leptos::prelude::*;
use std::collections::HashSet;

#[component]
pub fn GrammarMobileOverview(
    explanation: Memo<Option<String>>,
    how_to_form: Memo<Option<String>>,
    examples: Memo<Option<String>>,
    nuances: Memo<Option<String>>,
    pro_tip: Memo<Option<String>>,
    related_patterns: Memo<Option<String>>,
    #[prop(into)] explanation_title: Signal<String>,
    #[prop(into)] how_to_form_title: Signal<String>,
    #[prop(into)] examples_title: Signal<String>,
    #[prop(into)] nuances_title: Signal<String>,
    #[prop(into)] pro_tip_title: Signal<String>,
    #[prop(into)] related_title: Signal<String>,
    known_kanji: HashSet<char>,
) -> impl IntoView {
    let known_kanji_stored = StoredValue::new(known_kanji);

    view! {
        <Show when=move || explanation.get().is_some_and(|s| !s.is_empty())>
            <div class="grammar-detail-section">
                <div class="grammar-detail-section-card">
                    <div class="grammar-detail-section-title">{explanation_title}</div>
                    <MarkdownText
                        content=Signal::derive(move || explanation.get().unwrap_or_default())
                        known_kanji=known_kanji_stored.get_value()
                    />
                </div>
            </div>
        </Show>

        <Show when=move || how_to_form.get().is_some_and(|s| !s.is_empty())>
            <div class="grammar-detail-section">
                <div class="grammar-detail-section-card">
                    <div class="grammar-detail-section-title">{how_to_form_title}</div>
                    <MarkdownText
                        content=Signal::derive(move || how_to_form.get().unwrap_or_default())
                        known_kanji=known_kanji_stored.get_value()
                    />
                </div>
            </div>
        </Show>

        <Show when=move || examples.get().is_some_and(|s| !s.is_empty())>
            <div class="grammar-detail-section">
                <div class="grammar-detail-section-card">
                    <div class="grammar-detail-section-title">{examples_title}</div>
                    <MarkdownText
                        content=Signal::derive(move || examples.get().unwrap_or_default())
                        known_kanji=known_kanji_stored.get_value()
                    />
                </div>
            </div>
        </Show>

        <Show when=move || nuances.get().is_some_and(|s| !s.is_empty())>
            <div class="grammar-detail-section">
                <div class="grammar-detail-section-card">
                    <div class="grammar-detail-section-title">{nuances_title}</div>
                    <MarkdownText
                        content=Signal::derive(move || nuances.get().unwrap_or_default())
                        known_kanji=known_kanji_stored.get_value()
                    />
                </div>
            </div>
        </Show>

        <Show when=move || pro_tip.get().is_some_and(|s| !s.is_empty())>
            <div class="grammar-detail-section">
                <div class="grammar-detail-section-card">
                    <div class="grammar-detail-section-title">{pro_tip_title}</div>
                    <MarkdownText
                        content=Signal::derive(move || pro_tip.get().unwrap_or_default())
                        known_kanji=known_kanji_stored.get_value()
                    />
                </div>
            </div>
        </Show>

        <Show when=move || related_patterns.get().is_some_and(|s| !s.is_empty())>
            <div class="grammar-detail-section">
                <div class="grammar-detail-section-card">
                    <div class="grammar-detail-section-title">{related_title}</div>
                    <MarkdownText
                        content=Signal::derive(move || related_patterns.get().unwrap_or_default())
                        known_kanji=known_kanji_stored.get_value()
                    />
                </div>
            </div>
        </Show>
    }
}
