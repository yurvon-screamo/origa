pub mod hardest_words;
pub mod overdue;
pub mod pace;
pub mod panel_frame;
pub mod panels_data;
pub mod time_of_day;
pub mod week_verdict;

pub use panels_data::{InsightsPanels, compute_insights_panels};

use hardest_words::HardestWordsPanel;
use leptos::prelude::*;
use overdue::OverduePanel;
use pace::PacePanel;
use time_of_day::TimeOfDayPanel;
use week_verdict::WeekVerdictPanel;

use crate::i18n::{td_string, use_i18n};

/// Секция панелей-инсайтов на home (#749): пять вербализованных панелей —
/// вопрос, фраза-вердикт, раскрываемые доказательства. Все данные бесплатны.
/// Панели вычисляются на paint-путях content.rs и приходят готовым снимком.
#[component]
pub fn InsightsSection(
    panels: RwSignal<InsightsPanels>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let i18n = use_i18n();

    let section_prefix = move || {
        let val = test_id.get();
        if val.is_empty() {
            "home-insights".to_string()
        } else {
            val
        }
    };

    view! {
        <section data-testid=section_prefix>
            <p class="text-xs text-[var(--fg-muted)] uppercase tracking-widest mb-4">
                {move || {
                    let locale = i18n.get_locale();
                    td_string!(locale, insights.section_title)
                }}
            </p>

            <div class="grid grid-cols-1 gap-6 lg:grid-cols-2 lg:gap-8">
                <WeekVerdictPanel
                    verdict=Signal::derive(move || panels.get().week.clone())
                    test_id=Signal::derive(move || format!("{}-insights-week-verdict", section_prefix()))
                />
                <PacePanel
                    pace=Signal::derive(move || panels.get().pace)
                    test_id=Signal::derive(move || format!("{}-insights-pace", section_prefix()))
                />
                <TimeOfDayPanel
                    insight=Signal::derive(move || panels.get().time_of_day)
                    test_id=Signal::derive(move || format!("{}-insights-time-of-day", section_prefix()))
                />
                <OverduePanel
                    insight=Signal::derive(move || panels.get().overdue)
                    test_id=Signal::derive(move || format!("{}-insights-overdue", section_prefix()))
                />
                <div class="lg:col-span-2">
                    <HardestWordsPanel
                        insight=Signal::derive(move || panels.get().hardest_words)
                        test_id=Signal::derive(move || {
                            format!("{}-insights-hardest-words", section_prefix())
                        })
                    />
                </div>
            </div>
        </section>
    }
}
