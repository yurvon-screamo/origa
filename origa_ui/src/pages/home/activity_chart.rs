use super::dashboard_stats::ActivityDataPoint;
use crate::i18n::{t, td_string, use_i18n};
use crate::ui_components::{Card, ChartLine, MultiLineChart, Text, TextSize, TypographyVariant};
use leptos::prelude::*;
use origa::domain::RatingRatio;

#[component]
pub fn ActivityChart(
    chart_data: Signal<Vec<ActivityDataPoint>>,
    rating_ratio: Signal<Option<RatingRatio>>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let i18n = use_i18n();

    let ratio_color = Signal::derive(move || {
        rating_ratio
            .get()
            .map(|r| {
                if r.percentage > 60 {
                    "var(--success)"
                } else {
                    "var(--error)"
                }
            })
            .unwrap_or("var(--fg-muted)")
    });

    let ratio_pct = Signal::derive(move || rating_ratio.get().map(|r| r.percentage).unwrap_or(0));

    let has_ratio = Signal::derive(move || rating_ratio.get().is_some());

    let lines = Signal::derive(move || {
        let data = chart_data.get();
        let locale = i18n.get_locale();

        let learned_label = td_string!(locale, home.line_learned).to_string();
        let in_progress_label = td_string!(locale, home.line_in_progress).to_string();
        let new_label = td_string!(locale, home.line_new).to_string();
        let difficult_label = td_string!(locale, home.line_difficult).to_string();

        vec![
            ChartLine {
                data: data
                    .iter()
                    .map(|d| (d.date_label.clone(), d.learned))
                    .collect(),
                color: "var(--accent-sage)".to_string(),
                label: learned_label,
            },
            ChartLine {
                data: data
                    .iter()
                    .map(|d| (d.date_label.clone(), d.in_progress))
                    .collect(),
                color: "var(--accent-olive)".to_string(),
                label: in_progress_label,
            },
            ChartLine {
                data: data
                    .iter()
                    .map(|d| (d.date_label.clone(), d.new_count))
                    .collect(),
                color: "var(--accent-gold)".to_string(),
                label: new_label,
            },
            ChartLine {
                data: data
                    .iter()
                    .map(|d| (d.date_label.clone(), d.difficult))
                    .collect(),
                color: "var(--accent-terracotta)".to_string(),
                label: difficult_label,
            },
        ]
    });

    let has_enough_data = move || chart_data.get().len() >= 2;

    let is_mobile = RwSignal::new(false);
    Effect::new(move |_| {
        let width = web_sys::window()
            .and_then(|w| w.inner_width().ok())
            .and_then(|v: wasm_bindgen::JsValue| v.as_f64())
            .unwrap_or(800.0);
        is_mobile.set(width < 768.0);
    });

    let empty_text =
        Signal::derive(move || td_string!(i18n.get_locale(), home.no_data).to_string());

    view! {
        <Card shadow=true class=Signal::derive(|| "p-6 h-full".to_string()) test_id=test_id>
            <div class="flex items-center justify-between">
                <Text
                    size=TextSize::Small
                    variant=TypographyVariant::Muted
                    uppercase=true
                    tracking_widest=true
                >
                    {t!(i18n, home.activity_30days)}
                </Text>
                <Show when=move || has_ratio.get()>
                    <div
                        class="flex items-center gap-1.5 px-2.5 py-1"
                        style="background: var(--bg-aged);"
                        title=move || {
                            rating_ratio.get().map(|r| {
                                format!(
                                    "Easy/Good: {} · Hard/Again: {}",
                                    r.positive_count, r.negative_count
                                )
                            }).unwrap_or_default()
                        }
                    >
                        <span
                            class="inline-block"
                            style=move || {
                                let color = ratio_color.get();
                                format!("width:6px;height:6px;background:{}", color)
                            }
                        ></span>
                        <span class="font-mono text-[11px] text-[var(--fg-muted)]">
                            {t!(i18n, home.rating_accuracy)}
                        </span>
                        <span
                            class="font-mono text-[13px]"
                            style=move || format!("color:{}", ratio_color.get())
                        >
                            {move || format!("{}%", ratio_pct.get())}
                        </span>
                    </div>
                </Show>
            </div>

            <div class="mt-4 flex-1 flex flex-col min-h-0">
                <Show when=move || has_enough_data()>
                    <div class="flex-1 min-h-0">
                        // Mobile: viewBox 400x280 → at 400px container scale ~1x, font 12→12px
                        <Show when=move || is_mobile.get()>
                            <MultiLineChart
                                lines=lines
                                width=400
                                height=280
                                empty_text=empty_text
                            />
                        </Show>
                        // Desktop: viewBox 600x300 → at 670px container scale ~1.1x, font 12→13px
                        <Show when=move || !is_mobile.get()>
                            <MultiLineChart
                                lines=lines
                                width=600
                                height=300
                                empty_text=empty_text
                            />
                        </Show>
                    </div>
                </Show>

                <Show when=move || !has_enough_data()>
                    <div class="flex items-center justify-center flex-1">
                        <Text size=TextSize::Small variant=TypographyVariant::Muted>
                            {t!(i18n, home.no_data)}
                        </Text>
                    </div>
                </Show>
            </div>
        </Card>
    }
}
