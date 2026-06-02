use crate::ui_components::{FuriganaText, Tag};
use leptos::prelude::*;
use std::collections::HashSet;

#[component]
pub fn GrammarDetailHeroCard(
    title_stored: StoredValue<String>,
    short_description: Memo<String>,
    #[prop(into)] tag_variant: Signal<crate::ui_components::TagVariant>,
    #[prop(into)] tag_label: Signal<String>,
    known_kanji: HashSet<char>,
) -> impl IntoView {
    let known_kanji_stored = StoredValue::new(known_kanji);

    view! {
        <div class="grammar-detail-hero-card">
            <div class="grammar-detail-hero-header">
                <div class="grammar-detail-hero-form">
                    <FuriganaText
                        text=title_stored.get_value()
                        known_kanji=known_kanji_stored.get_value()
                    />
                </div>
                <Show when=move || !short_description.get().is_empty()>
                    <div class="grammar-detail-hero-meaning">{short_description}</div>
                </Show>
                <div class="grammar-detail-hero-badge">
                    <Tag variant=tag_variant>{tag_label}</Tag>
                </div>
            </div>
        </div>
    }
}
