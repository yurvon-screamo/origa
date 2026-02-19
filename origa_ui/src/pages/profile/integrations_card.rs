use super::LabeledInput;
use crate::ui_components::{Card, Heading, HeadingLevel};
use leptos::prelude::*;

#[component]
pub fn IntegrationsCard(duolingo_input: RwSignal<String>) -> impl IntoView {
    view! {
        <Card>
            <div class="space-y-4">
                <Heading level={HeadingLevel::H2}>
                    "Интеграции"
                </Heading>

                <div class="space-y-4">
                    <LabeledInput
                        label="Duolingo JWT Token".to_string()
                        value={duolingo_input}
                    />
                </div>
            </div>
        </Card>
    }
}
