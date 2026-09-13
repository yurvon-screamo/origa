//! Load-error screen for a full critical startup failure (ADR-053).
//!
//! Shown only when EVERY critical resource failed and the device has
//! nothing cached to fall back on (first offline launch, evicted
//! cache). A partial failure never blocks the app — it keeps running
//! with what loaded.

use crate::i18n::{t, use_i18n};
use crate::ui_components::{Button, ButtonVariant, Text, TextSize, TypographyVariant};
use leptos::prelude::*;

#[component]
pub fn LoadErrorScreen(#[prop(into)] on_retry: Callback<()>) -> impl IntoView {
    let i18n = use_i18n();

    view! {
        <div class="loading-overlay anima-page-fade" data-testid="app-load-error">
            <div class="load-error-content">
                <Text size=TextSize::Large variant=TypographyVariant::Primary>
                    {t!(i18n, ui.load_error.title)}
                </Text>
                <div class="mt-4 max-w-md text-center">
                    <Text size=TextSize::Default variant=TypographyVariant::Muted>
                        {t!(i18n, ui.load_error.body)}
                    </Text>
                </div>
                <div class="mt-8">
                    <Button
                        variant=ButtonVariant::Filled
                        test_id="app-load-error-retry"
                        on_click=Callback::new(move |_| on_retry.run(()))
                    >
                        {t!(i18n, ui.load_error.retry)}
                    </Button>
                </div>
            </div>
        </div>
    }
}
