use crate::i18n::{t, use_i18n};
use crate::ui_components::{Button, ButtonVariant, Heading, HeadingLevel, Tooltip};
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

#[component]
pub fn PhrasesHeader() -> impl IntoView {
    let i18n = use_i18n();
    let navigate = use_navigate();

    view! {
        <div class="flex flex-wrap justify-between items-center gap-4 mb-6">
            <div class="flex items-center gap-2">
                <Heading level=HeadingLevel::H1 test_id="phrases-title">
                    {t!(i18n, home.phrases)}
                </Heading>
                <Tooltip
                    text=Signal::derive(move || {
                        i18n.get_keys().phrases().hint().inner().to_string()
                    })
                    test_id=Signal::derive(|| "phrases-info-tooltip".to_string())
                >
                    <span class="inline-flex items-center justify-center w-5 h-5 rounded-full bg-[var(--bg-secondary)] text-[var(--fg-muted)] text-xs cursor-help" data-testid="phrases-info-icon">
                        "i"
                    </span>
                </Tooltip>
            </div>
            <div class="flex items-center gap-2 sm:gap-4">
                <Button
                    variant=ButtonVariant::Ghost
                    test_id="phrases-back-btn"
                    on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                        navigate("/home", Default::default());
                    })
                >
                    {t!(i18n, common.back)}
                </Button>
            </div>
        </div>
    }
}
