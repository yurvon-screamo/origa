use crate::ui_components::{Card, DisplayText, Tag, TagVariant};
use leptos::prelude::*;
use origa::domain::JftCoverage;

/// Home-page readiness bar for the JFT-Basic track (issue #760): a single
/// progress row for the learner's current band along the Irodori course
/// progression (A1 → A2.1 → A2, pass level), mirroring the JLPT progress
/// card's header. Rendered only when the user has at least one imported
/// Irodori lesson (visibility is gated by the caller) and the CDN catalog
/// loaded.
#[component]
pub fn JftReadinessCard(
    coverage: Signal<JftCoverage>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let current = Signal::derive(move || coverage.get().current().clone());
    let stamp = Signal::derive(move || format!("JFT-Basic {}", current.get().band.code()));
    let width = Signal::derive(move || format!("width: {:.0}%", current.get().percentage()));

    view! {
        <Card
            shadow=Signal::from(true)
            class=Signal::derive(|| "p-4 sm:p-6 lg:p-8".to_string())
            test_id=test_id.clone()
        >
            <div class="flex items-center gap-2 sm:gap-4" data-testid=Signal::derive(move || format!("{}-progress", test_id.get()))>
                <Tag
                    variant=Signal::from(TagVariant::Filled)
                    test_id=Signal::derive(move || format!("{}-stamp", test_id.get()))
                >
                    {stamp}
                </Tag>
                <div class="flex-1 progress-track">
                    <div class="progress-fill" style=width></div>
                </div>
                <DisplayText
                    class=Signal::derive(|| "text-sm".to_string())
                    test_id=Signal::derive(move || format!("{}-pct", test_id.get()))
                >
                    {move || format!("{:.0}%", current.get().percentage())}
                </DisplayText>
            </div>
        </Card>
    }
}
