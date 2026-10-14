pub mod hardest_words;
pub mod overdue;
pub mod pace;
pub mod panel_frame;
pub mod panels_data;
pub mod time_of_day;
pub mod week;

pub use panels_data::{InsightsPanels, compute_insights_panels};

use hardest_words::{words_label, words_subline_text, words_verdict_text};
use leptos::prelude::*;
use overdue::{overdue_label, overdue_subline_text, overdue_verdict_text};
use pace::{pace_label, pace_subline_text, pace_verdict_text};
use time_of_day::{time_label, time_subline_text, time_verdict_text};
use week::{verdict_label, week_subline_text, week_verdict_class, week_verdict_text};

use crate::i18n::{td_string, use_i18n};

/// Секция «Наблюдения» на home (#749): ruled-ledger из пяти
/// вербализованных строк — метка категории + одна фраза-вердикт.
/// Переключатель Неделя/Месяц меняет окно главного вердикта.
#[component]
pub fn InsightsSection(
    panels: RwSignal<InsightsPanels>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let i18n = use_i18n();
    let month_mode = RwSignal::new(false);

    let section_prefix = move || {
        let val = test_id.get();
        if val.is_empty() {
            "home-insights".to_string()
        } else {
            val
        }
    };

    // Вердикт выбранного периода: неделя (7д vs 30д) или месяц (30д vs 30д).
    let verdict_phrase = Signal::derive(move || {
        let data = panels.get();
        let verdict = if month_mode.get() {
            data.month.as_ref()
        } else {
            data.week.as_ref()
        };
        week_verdict_text(i18n, verdict)
    });

    view! {
        <section data-testid=move || test_id.get()>
            <div class="flex items-center justify-between mb-4">
                <p class="text-xs text-[var(--fg-muted)] uppercase tracking-widest">
                    {move || {
                        let locale = i18n.get_locale();
                        td_string!(locale, insights.section_title)
                    }}
                </p>

                <div
                    class="flex border border-[var(--border-dark)] font-mono text-xs"
                    data-testid=move || format!("{}-period", section_prefix())
                >
                    <button
                        class=move || period_chip_class(!month_mode.get())
                        on:click=move |_| month_mode.set(false)
                        data-testid=move || format!("{}-period-week", section_prefix())
                    >
                        {move || {
                            let locale = i18n.get_locale();
                            td_string!(locale, insights.period_week)
                        }}
                    </button>
                    <button
                        class=move || period_chip_class(month_mode.get())
                        on:click=move |_| month_mode.set(true)
                        data-testid=move || format!("{}-period-month", section_prefix())
                    >
                        {move || {
                            let locale = i18n.get_locale();
                            td_string!(locale, insights.period_month)
                        }}
                    </button>
                </div>
            </div>

            <div class="card card-shadow">
                <LedgerRow
                    label=Signal::derive(move || verdict_label(i18n))
                    phrase=verdict_phrase
                    phrase_class=Signal::derive(move || {
                        let data = panels.get();
                        week_verdict_class(data.week.as_ref()).to_string()
                    })
                    subline=Signal::derive(move || {
                        let data = panels.get();
                        week_subline_text(i18n, data.week.as_ref(), data.month.as_ref())
                    })
                    row_test_id=Signal::derive(move || format!("{}-week-verdict", section_prefix()))
                />
                <LedgerRow
                    label=Signal::derive(move || pace_label(i18n))
                    phrase=Signal::derive(move || {
                        pace_verdict_text(i18n, panels.get().pace.as_ref())
                    })
                    phrase_class=Signal::derive(|| "heading-h3 text-[var(--fg-black)]".to_string())
                    subline=Signal::derive(move || {
                        pace_subline_text(i18n, panels.get().pace.as_ref())
                    })
                    row_test_id=Signal::derive(move || format!("{}-pace", section_prefix()))
                />
                <LedgerRow
                    label=Signal::derive(move || time_label(i18n))
                    phrase=Signal::derive(move || {
                        time_verdict_text(i18n, panels.get().time_of_day.as_ref())
                    })
                    phrase_class=Signal::derive(|| "heading-h3 text-[var(--fg-black)]".to_string())
                    subline=Signal::derive(move || {
                        time_subline_text(i18n, panels.get().time_of_day.as_ref())
                    })
                    row_test_id=Signal::derive(move || format!("{}-time-of-day", section_prefix()))
                />
                <LedgerRow
                    label=Signal::derive(move || overdue_label(i18n))
                    phrase=Signal::derive(move || {
                        overdue_verdict_text(i18n, panels.get().overdue.as_ref())
                    })
                    phrase_class=Signal::derive(|| "heading-h3 text-[var(--fg-black)]".to_string())
                    subline=Signal::derive(move || {
                        overdue_subline_text(i18n, panels.get().overdue.as_ref())
                    })
                    row_test_id=Signal::derive(move || format!("{}-overdue", section_prefix()))
                />
                <LedgerRow
                    label=Signal::derive(move || words_label(i18n))
                    phrase=Signal::derive(move || {
                        words_verdict_text(i18n, panels.get().hardest_words.as_ref())
                    })
                    phrase_class=Signal::derive(|| "heading-h3 text-[var(--fg-black)]".to_string())
                    subline=Signal::derive(move || {
                        words_subline_text(i18n, panels.get().hardest_words.as_ref())
                    })
                    row_test_id=Signal::derive(move || format!("{}-hardest-words", section_prefix()))
                />
            </div>
        </section>
    }
}

/// Класс чипа переключателя периода: активный — залит чернилами.
fn period_chip_class(active: bool) -> &'static str {
    if active {
        "px-3 py-1 bg-[var(--fg-black)] text-[var(--bg-paper)] cursor-pointer"
    } else {
        "px-3 py-1 text-[var(--fg-black)] cursor-pointer"
    }
}

/// Одна строка ruled-ledger: mono-метка слева, фраза serif справа.
#[component]
fn LedgerRow(
    #[prop(into)] label: Signal<String>,
    #[prop(into)] phrase: Signal<String>,
    #[prop(optional, into)] phrase_class: Signal<String>,
    /// Дополнительная mono-строка под фразой (сводка обоих окон).
    #[prop(optional, into)]
    subline: Signal<String>,
    #[prop(optional, into)] row_test_id: Signal<String>,
) -> impl IntoView {
    view! {
        <div
            class="flex items-baseline gap-4 px-6 py-5 border-b border-[var(--border-light)] last:border-b-0"
            data-testid=move || row_test_id.get()
        >
            <span class="shrink-0 w-28 text-xs text-[var(--fg-muted)] uppercase tracking-widest font-mono">
                {move || label.get()}
            </span>
            <div class="flex flex-col gap-1">
                <span class=move || phrase_class.get()>
                    {move || phrase.get()}
                </span>
                <Show when=move || !subline.get().is_empty()>
                    <span class="font-mono text-xs text-[var(--fg-muted)]">
                        {move || subline.get()}
                    </span>
                </Show>
            </div>
        </div>
    }
}
