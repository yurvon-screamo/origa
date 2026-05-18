use super::dashboard_stats::ActivityDataPoint;
use crate::i18n::{t, td_string, use_i18n};
use crate::ui_components::{Card, ChartLine, MultiLineChart, Text, TextSize, TypographyVariant};
use leptos::prelude::*;

#[component]
pub fn ActivityChart(
    chart_data: Signal<Vec<ActivityDataPoint>>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let i18n = use_i18n();

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

    view! {
        <Card shadow=true class=Signal::derive(|| "p-6 h-full".to_string()) test_id=test_id>
            <Text
                size=TextSize::Small
                variant=TypographyVariant::Muted
                uppercase=true
                tracking_widest=true
            >
                {t!(i18n, home.activity_30days)}
            </Text>

            <div class="mt-4 flex-1 flex flex-col min-h-0">
                <Show when=move || has_enough_data()>
                    <div class="flex-1 min-h-0">
                        <MultiLineChart
                            lines=lines
                            width=400
                            height=200
                            class=Signal::derive(|| "h-full".to_string())
                            empty_text=Signal::derive(move || td_string!(i18n.get_locale(), home.no_data).to_string())
                        />
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
