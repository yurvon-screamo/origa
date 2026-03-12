use crate::ui_components::{Card, Heading, HeadingLevel, Text, TextSize};
use crate::version;
use leptos::prelude::*;

#[component]
pub fn SettingsCard() -> impl IntoView {
    view! {
        <Card>
            <div class="space-y-4">
                <Heading level={HeadingLevel::H2}>
                    "Настройки приложения"
                </Heading>

                <div class="space-y-2">
                    <Text size={TextSize::Large}>
                        "О приложении"
                    </Text>
                    <div class="space-y-1 text-sm text-[var(--fg-muted)]">
                        <div class="flex gap-2">
                            <span>"Версия:"</span>
                            <span class="font-mono">{version::VERSION}</span>
                        </div>
                        <div class="flex gap-2">
                            <span>"Commit:"</span>
                            <span class="font-mono">{version::COMMIT}</span>
                        </div>
                        <div class="flex gap-2">
                            <span>"Дата сборки:"</span>
                            <span class="font-mono">{version::BUILD_DATE}</span>
                        </div>
                    </div>
                </div>
            </div>
        </Card>
    }
}
