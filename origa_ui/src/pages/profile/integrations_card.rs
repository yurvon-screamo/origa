use crate::ui_components::{Card, Heading, HeadingLevel};
use leptos::prelude::*;

#[component]
pub fn IntegrationsCard() -> impl IntoView {
    view! {
        <Card>
            <div class="space-y-4">
                <Heading level={HeadingLevel::H2}>
                    "Интеграции"
                </Heading>
            </div>
        </Card>
    }
}
