use crate::i18n::{t, use_i18n};
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
    let i18n = use_i18n();

    let band_a1 = Signal::derive(move || coverage.get().bands()[0].clone());
    let band_a21 = Signal::derive(move || coverage.get().bands()[1].clone());
    let band_a2 = Signal::derive(move || coverage.get().bands()[2].clone());

    let header_test_id = move || {
        let val = test_id.get();
        if val.is_empty() {
            None
        } else {
            Some(format!("{val}-header"))
        }
    };

    view! {
        <Card
            shadow=Signal::from(true)
            class=Signal::derive(|| "p-4 sm:p-6 lg:p-8".to_string())
            test_id=test_id
        >
            <div class="flex flex-wrap items-baseline gap-x-3 gap-y-1" data-testid=header_test_id>
                <Tag variant=Signal::from(TagVariant::Filled)>"JFT-Basic"</Tag>
                <Text size=Signal::from(TextSize::Small) variant=Signal::from(TypographyVariant::Muted)>
                    {t!(i18n, home.jft_readiness.subtitle)}
                </Text>
            </div>

            <div class="space-y-3 mt-5">
                <BandRow band=band_a1 label_key=BandLabel::A1 />
                <BandRow band=band_a21 label_key=BandLabel::A21 />
                <BandRow band=band_a2 label_key=BandLabel::A2 />
            </div>
        </Card>
    }
}

/// Localized band label; the A2 row doubles as the pass-level marker
/// (200/250), per the approved copy decision.
#[derive(Clone, Copy)]
enum BandLabel {
    A1,
    A21,
    A2,
}

#[component]
fn BandRow(band: Signal<BandCoverage>, label_key: BandLabel) -> impl IntoView {
    let i18n = use_i18n();

    let label = Signal::derive(move || {
        let keys = i18n.get_keys().home().jft_readiness();
        match label_key {
            BandLabel::A1 => keys.a1().inner().to_string(),
            BandLabel::A21 => keys.a2_1().inner().to_string(),
            BandLabel::A2 => keys.a2().inner().to_string(),
        }
    });

    let band_code = move || band.get().band.code().to_string();
    let summary = Signal::derive(move || {
        let b = band.get();
        format!("{}/{} ({:.0}%)", b.covered, b.total, b.percentage())
    });
    let width = Signal::derive(move || format!("width: {:.0}%", band.get().percentage()));

    view! {
        <div
            class="grid grid-cols-[3.5rem_1fr_auto] items-center gap-3"
            data-testid=move || format!("home-jft-band-{}", band_code())
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
