use crate::i18n::*;
use crate::ui_components::legal_links;
use leptos::prelude::*;

// Plain function returning AnyView (not `#[component]`) — see legal_links.rs
// for the recursion_limit rationale.
pub fn legal_card(test_id: Signal<String>) -> AnyView {
    let i18n = use_i18n();
    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    view! {
        <div class="legal-card" data-testid=test_id_val>
            <div class="legal-card__title">{t!(i18n, profile.legal)}</div>
            {legal_links(Signal::derive(|| "profile-legal-links".to_string()))}
        </div>
    }
    .into_any()
}
