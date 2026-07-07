use crate::core::tauri;
use crate::i18n::{t, use_i18n};
use leptos::prelude::*;

const PRIVACY_URL: &str = "https://origa.uwuwu.net/privacy";
const TERMS_URL: &str = "https://origa.uwuwu.net/terms";

// Plain function returning AnyView (not `#[component]`) so the call site
// inserts an already-erased view. The bin crate inherits the default
// recursion_limit of 128 and monomorphises the whole `<App/>` tree; a
// `#[component]` wrapper would add a component-node type layer and tip an
// unrelated deep component over the limit (ADR-027). AnyView keeps the
// parent's type-depth flat regardless of the internal button attributes.
pub fn legal_links(test_id: Signal<String>) -> AnyView {
    let i18n = use_i18n();
    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    view! {
        <div class="legal-links" data-testid=test_id_val>
            <button
                type="button"
                class="legal-links__item"
                data-testid="legal-links-privacy"
                on:click=move |_: leptos::ev::MouseEvent| {
                    tauri::open_url_external(PRIVACY_URL);
                }
            >
                {t!(i18n, legal.privacy_policy)}
            </button>
            <button
                type="button"
                class="legal-links__item"
                data-testid="legal-links-terms"
                on:click=move |_: leptos::ev::MouseEvent| {
                    tauri::open_url_external(TERMS_URL);
                }
            >
                {t!(i18n, legal.terms_of_service)}
            </button>
        </div>
    }
    .into_any()
}
