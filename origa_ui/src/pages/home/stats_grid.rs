use super::{LessonButtonsCard, QuickStatCard, StatMetric};
use super::{PrimaryStats, SecondaryStats, format_delta, format_number};
use crate::i18n::{t, use_i18n};
use crate::ui_components::{Button, ButtonVariant};
use leptos::prelude::*;

#[component]
pub fn StatsGrid(
    primary: Signal<PrimaryStats>,
    secondary: Signal<SecondaryStats>,
    completion_badge: Signal<Option<String>>,
    // Send + Sync требуются leptos view! для children closures
    open_history: impl Fn(StatMetric) -> Callback<()> + Send + Sync + 'static,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let i18n = use_i18n();
    let show_details = RwSignal::new(false);

    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    let toggle_class =
        Signal::derive(move || if show_details.get() { "rotate-180" } else { "" }.to_string());

    view! {
        <div data-testid=test_id_val>
            <div class="flex flex-col md:flex-row gap-6 mb-6">
                <div class="md:w-1/4">
                    <LessonButtonsCard test_id=Signal::derive(|| "lesson-buttons".to_string()) />
                </div>
                <div class="md:w-3/4">
                    <div class="grid grid-cols-2 sm:grid-cols-4 gap-4">
                        <QuickStatCard
                            title=Signal::derive(move || i18n.get_keys().home().learned().inner().to_string())
                            value=Signal::derive(move || format_number(primary.get().learned))
                            delta=Signal::derive(move || format_delta(primary.get().learned_delta))
                            on_card_click=open_history(StatMetric::Learned)
                            test_id=Signal::derive(|| "stat-learned".to_string())
                        />
                        <QuickStatCard
                            title=Signal::derive(move || i18n.get_keys().home().in_progress().inner().to_string())
                            value=Signal::derive(move || format_number(primary.get().in_progress))
                            delta=Signal::derive(move || format_delta(primary.get().in_progress_delta))
                            on_card_click=open_history(StatMetric::InProgress)
                            test_id=Signal::derive(|| "stat-in-progress".to_string())
                        />
                        <QuickStatCard
                            title=Signal::derive(move || i18n.get_keys().home().hard().inner().to_string())
                            value=Signal::derive(move || format_number(secondary.get().high_difficulty))
                            delta=Signal::derive(move || format_delta(secondary.get().high_difficulty_delta))
                            on_card_click=open_history(StatMetric::HighDifficulty)
                            test_id=Signal::derive(|| "stat-high-difficulty".to_string())
                        />
                        <QuickStatCard
                            title=Signal::derive(move || i18n.get_keys().home().new_items().inner().to_string())
                            value=Signal::derive(move || format_number(primary.get().new))
                            delta=Signal::derive(move || format_delta(primary.get().new_delta))
                            badge=completion_badge
                            on_card_click=open_history(StatMetric::New)
                            test_id=Signal::derive(|| "stat-new".to_string())
                        />
                    </div>
                </div>
            </div>

            <div class="mb-4">
                <Button
                    variant=Signal::from(ButtonVariant::Ghost)
                    on_click=Callback::new(move |_: leptos::ev::MouseEvent| show_details.update(|v| *v = !*v))
                    test_id=Signal::derive(|| "toggle-details".to_string())
                >
                    <span class="flex items-center gap-2">
                        {t!(i18n, home.detailed_stats)}
                        <span class=move || toggle_class.get()>"▼"</span>
                    </span>
                </Button>
            </div>

            <Show when=move || show_details.get()>
                <div class="grid grid-cols-2 sm:grid-cols-3 gap-4">
                    <QuickStatCard
                        title=Signal::derive(move || i18n.get_keys().home().positive().inner().to_string())
                        value=Signal::derive(move || format_number(secondary.get().positive))
                        delta=Signal::derive(move || format_delta(secondary.get().positive_delta))
                        on_card_click=open_history(StatMetric::PositiveRatings)
                        test_id=Signal::derive(|| "stat-positive".to_string())
                    />
                    <QuickStatCard
                        title=Signal::derive(move || i18n.get_keys().home().negative().inner().to_string())
                        value=Signal::derive(move || format_number(secondary.get().negative))
                        delta=Signal::derive(move || format_delta(secondary.get().negative_delta))
                        on_card_click=open_history(StatMetric::NegativeRatings)
                        test_id=Signal::derive(|| "stat-negative".to_string())
                    />
                    <QuickStatCard
                        title=Signal::derive(move || i18n.get_keys().home().total_ratings().inner().to_string())
                        value=Signal::derive(move || format_number(secondary.get().total_ratings))
                        delta=Signal::derive(move || format_delta(secondary.get().total_ratings_delta))
                        on_card_click=open_history(StatMetric::TotalRatings)
                        test_id=Signal::derive(|| "stat-total-ratings".to_string())
                    />
                </div>
            </Show>
        </div>
    }
}
