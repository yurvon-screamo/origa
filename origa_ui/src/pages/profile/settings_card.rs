use crate::core::version;
use crate::i18n::*;
use crate::ui_components::{Card, Text, TextSize};
use leptos::prelude::*;

#[component]
pub fn SettingsCard(#[prop(optional, into)] test_id: Signal<String>) -> impl IntoView {
    let i18n = use_i18n();

    view! {
        <Card test_id=test_id>
            <div class="space-y-4">
                <div class="space-y-2">
                    <Text size={TextSize::Large}>
                        {t!(i18n, profile.about_app)}
                    </Text>
                    <div class="space-y-1 text-sm text-[var(--fg-muted)]">
                        <div class="flex gap-2">
                            <span>{t!(i18n, profile.version)}</span>
                            <span class="font-mono">{version::VERSION}</span>
                        </div>
                        <div class="flex gap-2">
                            <span>{t!(i18n, profile.commit)}</span>
                            <span class="font-mono min-w-0 truncate">{version::COMMIT}</span>
                        </div>
                        <div class="flex gap-2">
                            <span>{t!(i18n, profile.build_date)}</span>
                            <span class="font-mono">{version::BUILD_DATE}</span>
                        </div>
                    </div>
                </div>
            </div>
        </Card>
    }
}
