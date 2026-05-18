use super::dashboard_stats::{CompletionForecast, TodayOverview};
use crate::i18n::{t, td_string, use_i18n};
use crate::ui_components::{Card, DisplayText, Text, TextSize, TypographyVariant};
use leptos::prelude::*;

#[component]
pub fn TodayOverviewCard(
    overview: Signal<TodayOverview>,
    forecast: Signal<CompletionForecast>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let i18n = use_i18n();

    let total = Signal::derive(move || overview.get().total());
    let new_count = Signal::derive(move || overview.get().new_count);
    let learned_count = Signal::derive(move || overview.get().learned_count);
    let in_progress_count = Signal::derive(move || overview.get().in_progress_count);
    let difficult_count = Signal::derive(move || overview.get().difficult_count);

    let max_count = Signal::derive(move || {
        let ov = overview.get();
        ov.new_count
            .max(ov.learned_count)
            .max(ov.in_progress_count)
            .max(ov.difficult_count)
            .max(1)
    });

    let progress_pct = move |count: usize| -> String {
        if count == 0 {
            "0%".to_string()
        } else {
            let pct = (count as f64 / max_count.get() as f64 * 100.0).min(100.0);
            format!("{:.0}%", pct)
        }
    };

    let days_label = Signal::derive(move || {
        let locale = i18n.get_locale();
        td_string!(locale, home.days_label)
    });

    view! {
        <Card shadow=true class=Signal::derive(|| "p-8 h-full".to_string()) test_id=test_id>
            <div class="flex flex-col h-full">
                <div class="flex items-baseline gap-3">
                    <Text
                        size=TextSize::Small
                        variant=TypographyVariant::Muted
                        uppercase=true
                        tracking_widest=true
                    >
                        {t!(i18n, home.today_overview)}
                    </Text>

                    <div class="flex items-baseline gap-1">
                        <DisplayText class=Signal::derive(|| "font-serif text-[32px] font-light leading-tight".to_string())>
                            {move || total.get().to_string()}
                        </DisplayText>

                        <Text size=TextSize::Small variant=TypographyVariant::Muted>
                            {t!(i18n, home.total_label)}
                        </Text>
                    </div>

                    <div class="ml-auto">
                        <Show when=move || forecast.get().days_remaining.is_some()>
                            <span class="font-mono text-[12px] text-[var(--fg-muted)]">
                                {move || {
                                    let fc = forecast.get();
                                    let days = fc.days_remaining.unwrap_or(0);
                                    format!(
                                        "~{} {} · {}",
                                        days,
                                        days_label.get(),
                                        fc.target_date_label
                                    )
                                }}
                            </span>
                        </Show>

                        <Show when=move || forecast.get().is_all_studied>
                            <span class="font-mono text-[12px] text-[var(--accent-olive)]">
                                {t!(i18n, home.all_studied)}
                            </span>
                        </Show>
                    </div>
                </div>

                <div class="status-grid mt-4">
                    <div class="status-card">
                        <span class="status-number">{move || new_count.get()}</span>
                        <span class="status-label">{t!(i18n, home.new_status)}</span>
                        <div class="status-progress">
                            <div
                                class="status-progress-fill"
                                style=move || format!("width: {}", progress_pct(new_count.get()))
                            ></div>
                        </div>
                    </div>

                    <div class="status-card">
                        <span class="status-number">{move || learned_count.get()}</span>
                        <span class="status-label">{t!(i18n, home.learned_status)}</span>
                        <div class="status-progress">
                            <div
                                class="status-progress-fill"
                                style=move || format!("width: {}", progress_pct(learned_count.get()))
                            ></div>
                        </div>
                    </div>

                    <div class="status-card">
                        <span class="status-number">{move || in_progress_count.get()}</span>
                        <span class="status-label">{t!(i18n, home.in_progress_status)}</span>
                        <div class="status-progress">
                            <div
                                class="status-progress-fill"
                                style=move || format!("width: {}", progress_pct(in_progress_count.get()))
                            ></div>
                        </div>
                    </div>

                    <div class="status-card status-card--difficult">
                        <span class="status-number">{move || difficult_count.get()}</span>
                        <span class="status-label">{t!(i18n, home.difficult_status)}</span>
                        <div class="status-progress">
                            <div
                                class="status-progress-fill"
                                style=move || format!("width: {}", progress_pct(difficult_count.get()))
                            ></div>
                        </div>
                    </div>
                </div>
            </div>
        </Card>
    }
}
