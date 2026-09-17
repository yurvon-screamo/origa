use crate::ui_components::{Tag, TagVariant};
use leptos::prelude::*;

/// Grammar badge shown on a lesson card once the answer is revealed.
/// The localized meaning leads; the bare Japanese pattern trails in
/// monospace — a bare pattern (使役形) alone reads as noise (#503 UX).
/// The Tag base style is uppercase mono with wide tracking, which is
/// unreadable for a sentence-length meaning: both spans reset it.
#[component]
pub fn GrammarInfoBadge(pattern: String, meaning: String) -> impl IntoView {
    view! {
        <Tag
            variant=Signal::derive(|| TagVariant::Default)
        >
            <span class="grammar-badge-meaning">{meaning}</span>
            <span class="grammar-badge-pattern">{pattern}</span>
        </Tag>
    }
}
