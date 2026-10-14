use crate::ui_components::{Card, Tag, TagVariant, Text, TextSize, TypographyVariant};
use leptos::prelude::*;
use origa::domain::{BandCoverage, JftCoverage};

/// Home-page readiness card for the JFT-Basic track (issue #760): Can-do
/// coverage per band along the Irodori course progression. Rendered only
/// when the user has at least one imported Irodori lesson (visibility is
/// gated by the caller) and the CDN catalog loaded.
#[component]
pub fn JftReadinessCard(
    coverage: Signal<JftCoverage>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let band_a1 = Signal::derive(move || coverage.get().bands()[0].clone());
    let band_a21 = Signal::derive(move || coverage.get().bands()[1].clone());
    let band_a2 = Signal::derive(move || coverage.get().bands()[2].clone());

    view! {
        <Card
            shadow=Signal::from(true)
            class=Signal::derive(|| "p-4 sm:p-6 lg:p-8".to_string())
            test_id=test_id
        >
            <div class="mb-4">
                <Tag variant=Signal::from(TagVariant::Filled)>"JFT-Basic"</Tag>
            </div>
            <div class="space-y-3">
                <BandRow band=band_a1 />
                <BandRow band=band_a21 />
                <BandRow band=band_a2 />
            </div>
        </Card>
    }
}

#[component]
fn BandRow(band: Signal<BandCoverage>) -> impl IntoView {
    let label = Signal::derive(move || band.get().band.code().to_string());

    // Dot-free slug: "A2.1" would break unquoted CSS attribute selectors.
    let band_slug = move || band.get().band.code().replace('.', "");
    let summary = Signal::derive(move || {
        let b = band.get();
        format!("{}/{} ({:.0}%)", b.covered, b.total, b.percentage())
    });
    let width = Signal::derive(move || format!("width: {:.0}%", band.get().percentage()));

    view! {
        <div
            class="grid grid-cols-[max-content_1fr_auto] items-center gap-3"
            data-testid=move || format!("home-jft-band-{}", band_slug())
        >
            <Text size=Signal::from(TextSize::Small)>{label}</Text>
            <div class="progress-track">
                <div class="progress-fill" style=width></div>
            </div>
            <Text
                size=Signal::from(TextSize::Small)
                variant=Signal::from(TypographyVariant::Muted)
                class=Signal::derive(|| "tabular-nums whitespace-nowrap".to_string())
            >
                {summary}
            </Text>
        </div>
    }
}
