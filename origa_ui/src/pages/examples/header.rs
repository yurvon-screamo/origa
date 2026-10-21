use crate::i18n::use_i18n;
use crate::ui_components::{PageHeader, Tooltip};
use leptos::prelude::*;

#[component]
pub fn ExamplesHeader() -> impl IntoView {
    let i18n = use_i18n();

    view! {
        <PageHeader
            back_path="".to_string()
            back_label=Signal::derive(move || i18n.get_keys().common().back().inner().to_string())
            title=Signal::derive(move || i18n.get_keys().examples().title().inner().to_string())
            test_id="examples"
        >
            <Tooltip
                text=Signal::derive(move || {
                    i18n.get_keys().examples().hint().inner().to_string()
                })
                test_id=Signal::derive(|| "examples-info-tooltip".to_string())
            >
                <span class="inline-flex items-center justify-center w-5 h-5 rounded-full bg-[var(--bg-aged)] text-[var(--fg-muted)] text-xs" data-testid="examples-info-icon">
                    "i"
                </span>
            </Tooltip>
        </PageHeader>
    }
}
