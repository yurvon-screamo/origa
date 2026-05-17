use super::dashboard_stats::TodayOverview;
use crate::i18n::{t, td_string, use_i18n};
use crate::ui_components::{Card, DisplayText, Text, TextSize, TypographyVariant};
use leptos::prelude::*;

#[component]
pub fn TodayOverviewCard(
    overview: Signal<TodayOverview>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let i18n = use_i18n();

    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    let total = Signal::derive(move || overview.get().total());
    let new_count = Signal::derive(move || overview.get().new_count);
    let learning_count = Signal::derive(move || overview.get().learning_count);
    let review_count = Signal::derive(move || overview.get().review_count);

    let progress_pct = Signal::derive(move || {
        let ov = overview.get();
        let t = ov.total().max(1);
        (ov.review_count as f64 / t as f64 * 100.0).min(100.0)
    });

    let has_new_items = Signal::derive(move || overview.get().new_count > 0);

    view! {
        <Card shadow=true class=Signal::derive(|| "p-8 h-full".to_string()) test_id=test_id>
            <div data-testid=test_id_val class="flex flex-col h-full">
                <Text
                    size=TextSize::Small
                    variant=TypographyVariant::Muted
                    uppercase=true
                    tracking_widest=true
                >
                    {t!(i18n, home.today_overview)}
                </Text>

                <div class="flex items-start gap-4 mt-3">
                    <div class="flex flex-col">
                        <DisplayText class=Signal::derive(|| "font-serif text-[48px] font-light leading-tight mb-1".to_string())>
                            {move || total.get().to_string()}
                        </DisplayText>
                        <Text
                            size=TextSize::Small
                            variant=TypographyVariant::Muted
                            uppercase=true
                        >
                            {move || {
                                let locale = i18n.get_locale();
                                if has_new_items.get() {
                                    td_string!(locale, home.today_study_items).to_string()
                                } else {
                                    td_string!(locale, home.today_review_items).to_string()
                                }
                            }}
                        </Text>
                    </div>

                    <div class="flex flex-col gap-1">
                        <div class="flex items-center gap-2">
                            <span class="font-mono text-[11px] uppercase text-[var(--fg-muted)]">
                                {t!(i18n, home.status_good)}
                            </span>
                            <span class="w-2 h-2 bg-[var(--accent-olive)]" aria-hidden="true"></span>
                        </div>
                        <span class="font-mono text-[11px] uppercase text-[var(--fg-light)]">
                            {t!(i18n, home.fsrs_status)}
                        </span>
                    </div>
                </div>

                <div class="progress-track mt-4">
                    <div
                        class="progress-fill"
                        style=move || format!("width: {:.0}%", progress_pct.get())
                    ></div>
                </div>

                <div class="border-t border-[var(--border-light)] my-4"></div>

                <div class="flex gap-6">
                    <div class="flex flex-col">
                        <span class="font-serif text-xl text-[var(--accent-terracotta)]">
                            {move || new_count.get().to_string()}
                        </span>
                        <span class="font-mono text-[11px] uppercase text-[var(--fg-muted)]">
                            {t!(i18n, home.new_status)}
                        </span>
                    </div>

                    <div class="flex flex-col">
                        <span class="font-serif text-xl text-[var(--accent-gold)]">
                            {move || learning_count.get().to_string()}
                        </span>
                        <span class="font-mono text-[11px] uppercase text-[var(--fg-muted)]">
                            {t!(i18n, home.learning_status)}
                        </span>
                    </div>

                    <div class="flex flex-col">
                        <span class="font-serif text-xl text-[var(--accent-sage)]">
                            {move || review_count.get().to_string()}
                        </span>
                        <span class="font-mono text-[11px] uppercase text-[var(--fg-muted)]">
                            {t!(i18n, home.review_status)}
                        </span>
                    </div>
                </div>
            </div>
        </Card>
    }
}
