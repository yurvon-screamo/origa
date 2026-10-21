//! Common header builder of the card list pages (phrases, examples): a
//! PageHeader with the section title and the info tooltip hint (#823).
//! A plain function — the page headers stay thin i18n adapters.

use crate::i18n::use_i18n;
use crate::ui_components::{PageHeader, Tooltip};
use leptos::prelude::*;

pub fn card_list_page_header(
    prefix: &'static str,
    title: impl Fn() -> String + Send + Sync + 'static,
    hint: impl Fn() -> String + Send + Sync + 'static,
) -> impl IntoView {
    let i18n = use_i18n();
    let back_label = move || i18n.get_keys().common().back().inner().to_string();
    let tooltip_text = hint;

    view! {
        <PageHeader
            back_path="".to_string()
            back_label=Signal::derive(back_label)
            title=Signal::derive(title)
            test_id=prefix
        >
            <Tooltip
                text=Signal::derive(tooltip_text)
                test_id=Signal::derive(move || format!("{prefix}-info-tooltip"))
            >
                <span class="inline-flex items-center justify-center w-5 h-5 rounded-full bg-[var(--bg-aged)] text-[var(--fg-muted)] text-xs" data-testid=move || format!("{prefix}-info-icon")>
                    "i"
                </span>
            </Tooltip>
        </PageHeader>
    }
}
