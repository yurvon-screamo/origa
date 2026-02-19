use crate::ui_components::{Card, Heading, HeadingLevel, Text, TextSize, Toggle};
use leptos::prelude::*;

#[component]
pub fn SettingsCard(reminders: RwSignal<bool>) -> impl IntoView {
    view! {
        <Card>
            <div class="space-y-4">
                <Heading level={HeadingLevel::H2}>
                    "Настройки приложения"
                </Heading>

                <div class="flex items-center justify-between">
                    <Text size={TextSize::Large}>
                        "Напоминания"
                    </Text>
                    <Toggle
                        checked={Signal::derive(move || reminders.get())}
                        on_change={Callback::new(move |_| {
                            reminders.update(|r| *r = !*r);
                        })}
                        label="".to_string()
                    />
                </div>
            </div>
        </Card>
    }
}
