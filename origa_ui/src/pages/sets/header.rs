use crate::i18n::use_i18n;
use crate::ui_components::PageHeader;
use leptos::prelude::*;

#[component]
pub fn SetsHeader() -> impl IntoView {
    let i18n = use_i18n();

    view! {
        <PageHeader
            back_path="/words".to_string()
            back_label=Signal::derive(move || i18n.get_keys().common().back().inner().to_string())
            title=Signal::derive(move || i18n.get_keys().sets().header().inner().to_string())
            test_id="sets"
        />
    }
}
