use super::dashboard_stats::CompletionForecast;
use crate::i18n::{t, td_string, use_i18n};
use crate::ui_components::{Card, Text, TextSize, TypographyVariant};
use leptos::prelude::*;

#[component]
pub fn CompletionForecastCard(
    forecast: Signal<CompletionForecast>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let i18n = use_i18n();

    let has_forecast = Signal::derive(move || forecast.get().days_remaining.is_some());
    let is_all_studied = Signal::derive(move || forecast.get().is_all_studied);
    let is_empty = Signal::derive(move || forecast.get().total_cards == 0);
    let has_any_data =
        Signal::derive(move || !is_empty.get() && (has_forecast.get() || is_all_studied.get()));

    let days = Signal::derive(move || forecast.get().days_remaining.unwrap_or(0));
    let target_date = Signal::derive(move || forecast.get().target_date_label.clone());
    let cards_per_day = Signal::derive(move || forecast.get().cards_per_day);
    let progress_pct = Signal::derive(move || forecast.get().progress_pct);
    let known_cards = Signal::derive(move || forecast.get().known_cards);
    let total_cards = Signal::derive(move || forecast.get().total_cards);

    view! {
        <Card shadow=true class="p-6" test_id=test_id>
            <Text size=TextSize::Small variant=TypographyVariant::Muted uppercase=true tracking_widest=true>
                {t!(i18n, home.completion_forecast)}
            </Text>

            <Show when=move || is_empty.get()>
                <div class="mt-3">
                    <Text size=TextSize::Small variant=TypographyVariant::Muted>
                        {t!(i18n, home.insufficient_data)}
                    </Text>
                </div>
            </Show>

            <Show when=move || has_any_data.get()>
                <div class="mt-4">
                    <Show when=move || is_all_studied.get()>
                        <div class="font-serif text-[32px] text-[var(--accent-olive)]">
                            {t!(i18n, home.all_studied)}
                        </div>
                    </Show>
                    <Show when=move || !is_all_studied.get() && has_forecast.get()>
                        <div class="font-serif text-[32px]">
                            {move || {
                                format!("~{} {}", days.get(), td_string!(i18n.get_locale(), home.days_label))
                            }}
                        </div>
                        <div class="font-mono text-[12px] text-[var(--fg-muted)] uppercase mt-1">
                            {move || target_date.get()}
                        </div>
                        <div class="font-mono text-[11px] text-[var(--fg-muted)] mt-1">
                            {move || {
                                let locale = i18n.get_locale();
                                format!("{} {:.0} {}",
                                    td_string!(locale, home.cards_per_day_label),
                                    cards_per_day.get(),
                                    td_string!(locale, home.per_day_label))
                            }}
                        </div>
                    </Show>

                    <div class="mt-4">
                        <div class="flex justify-between items-center mb-1">
                            <Text size=TextSize::Small variant=TypographyVariant::Muted>
                                {t!(i18n, home.progress_percent)}
                            </Text>
                            <Text size=TextSize::Small variant=TypographyVariant::Muted>
                                {move || format!("{:.0}%", progress_pct.get())}
                            </Text>
                        </div>
                        <div class="forecast-progress">
                            <div
                                class="forecast-progress-fill"
                                style=move || format!("width: {:.0}%", progress_pct.get())
                            ></div>
                        </div>
                        <div class="flex justify-between mt-1">
                            <span class="font-mono text-[10px] text-[var(--fg-light)]">
                                {move || {
                                    let locale = i18n.get_locale();
                                    format!(
                                        "{} {} {}",
                                        known_cards.get(),
                                        td_string!(locale, home.of_label),
                                        total_cards.get(),
                                    )
                                }}
                            </span>
                        </div>
                    </div>
                </div>
            </Show>

            <Show when=move || !is_empty.get() && !has_any_data.get()>
                <div class="mt-3">
                    <Text size=TextSize::Small variant=TypographyVariant::Muted>
                        {t!(i18n, home.insufficient_data)}
                    </Text>
                </div>
            </Show>
        </Card>
    }
}
