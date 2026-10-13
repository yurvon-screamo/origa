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

    view! {
        <section data-testid=move || test_id.get()>
            <p class="text-xs text-[var(--fg-muted)] uppercase tracking-widest mb-4">
                {move || {
                    let locale = i18n.get_locale();
                    td_string!(locale, insights.section_title)
                }}
            </p>

            <div class="grid grid-cols-1 gap-6 lg:grid-cols-2 lg:gap-8">
                <WeekVerdictPanel
                    verdict=Signal::derive(move || panels.get().week.clone())
                    test_id=Signal::derive(|| "insights-week-verdict".to_string())
                />
                <PacePanel
                    pace=Signal::derive(move || panels.get().pace)
                    test_id=Signal::derive(|| "insights-pace".to_string())
                />
                <TimeOfDayPanel
                    insight=Signal::derive(move || panels.get().time_of_day)
                    test_id=Signal::derive(|| "insights-time-of-day".to_string())
                />
                <OverduePanel
                    insight=Signal::derive(move || panels.get().overdue)
                    test_id=Signal::derive(|| "insights-overdue".to_string())
                />
                <div class="lg:col-span-2">
                    <HardestWordsPanel
                        insight=Signal::derive(move || panels.get().hardest_words)
                        test_id=Signal::derive(|| "insights-hardest-words".to_string())
                    />
                </div>
            </div>
        </section>
    }
}
