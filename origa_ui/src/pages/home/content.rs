use super::content_sync::run_sync;
use super::{HistoryModal, HomeSkeleton, JlptProgressCard, JlptSkeleton, StatMetric, StatsGrid};
use super::{PrimaryStats, SecondaryStats, calculate_stats};
use crate::repository::HybridUserRepository;
use crate::ui_components::{Text, TextSize, ToastContainer, ToastData, TypographyVariant};
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::domain::{DailyHistoryItem, JlptProgress};
use origa::traits::UserRepository;

#[component]
pub fn HomeContent(#[prop(optional, into)] test_id: Signal<String>) -> impl IntoView {
    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    let repository =
        use_context::<HybridUserRepository>().expect("repository context not provided");

    let stats = RwSignal::new(None::<(PrimaryStats, SecondaryStats)>);
    let history = RwSignal::new(Vec::<DailyHistoryItem>::new());
    let history_open = RwSignal::new(false);
    let selected_metric = RwSignal::new(StatMetric::TotalCards);
    let is_loading = RwSignal::new(true);
    let jlpt_progress = RwSignal::new(JlptProgress::new());
    let toasts: RwSignal<Vec<ToastData>> = RwSignal::new(Vec::new());
    let disposed = StoredValue::new(());

    let repo_sync = repository.clone();
    Effect::new(move |_| run_sync(repo_sync.clone(), disposed, toasts));

    let repo_for_init = repository.clone();
    Effect::new(move |_| {
        let repo = repo_for_init.clone();
        spawn_local(async move {
            match repo.get_current_user().await {
                Ok(Some(user)) => {
                    if disposed.is_disposed() {
                        return;
                    }
                    let history_items = user.knowledge_set().lesson_history().to_vec();
                    history.set(history_items.clone());
                    stats.set(Some(calculate_stats(&history_items)));
                    jlpt_progress.set(user.jlpt_progress().clone());
                    is_loading.set(false);
                },
                Ok(None) => {
                    tracing::warn!("Home: user not found");
                    stats.set(Some((PrimaryStats::default(), SecondaryStats::default())));
                    is_loading.set(false);
                },
                Err(e) => {
                    tracing::error!("Home: get_current_user error: {:?}", e);
                    stats.set(Some((PrimaryStats::default(), SecondaryStats::default())));
                    is_loading.set(false);
                },
            }
        });
    });

    let primary = Signal::derive(move || stats.get().map(|(p, _)| p).unwrap_or_default());
    let secondary = Signal::derive(move || stats.get().map(|(_, s)| s).unwrap_or_default());

    let open_history = move |metric: StatMetric| {
        Callback::new(move |_: ()| {
            selected_metric.set(metric);
            history_open.set(true);
        })
    };

    let close_history = Callback::new(move |_: ()| {
        history_open.set(false);
    });

    view! {
        <main class="flex-1" data-testid=test_id_val>
            <div class="w-full px-4 sm:px-6 lg:px-8 py-12">
                <div class="flex items-center justify-between mb-6">
                    <Text
                        size=TextSize::Small
                        variant=TypographyVariant::Muted
                        uppercase=true
                        tracking_widest=true
                        test_id=Signal::derive(move || {
                            let val = test_id.get();
                            if val.is_empty() { "home-stats-title".to_string() } else { val }
                        })
                    >
                        "Статистика"
                    </Text>
                </div>

                <Show
                    when=move || !is_loading.get()
                    fallback=move || view! { <JlptSkeleton /> }
                >
                    <JlptProgressCard
                        jlpt_progress=Signal::derive(move || jlpt_progress.get())
                        test_id=Signal::derive(move || {
                            let val = test_id.get();
                            if val.is_empty() {
                                "home-jlpt-progress".to_string()
                            } else {
                                format!("{}-jlpt-progress", val)
                            }
                        })
                    />
                </Show>

                <Show
                    when=move || !is_loading.get()
                    fallback=move || view! { <HomeSkeleton /> }
                >
                    <StatsGrid
                        primary=primary
                        secondary=secondary
                        open_history=open_history
                        test_id=Signal::derive(move || {
                            let val = test_id.get();
                            if val.is_empty() { "home-stats-grid".to_string() } else { val }
                        })
                    />
                </Show>
            </div>

            <HistoryModal
                is_open=Signal::derive(move || history_open.get())
                metric=Signal::derive(move || selected_metric.get())
                history=Signal::derive(move || history.get())
                on_close=close_history
                test_id=Signal::derive(move || {
                    let val = test_id.get();
                    if val.is_empty() {
                        "home-history-modal".to_string()
                    } else {
                        format!("{}-history-modal", val)
                    }
                })
            />

            <ToastContainer
                toasts=toasts
                duration_ms=5000
                test_id=Signal::derive(move || {
                    let val = test_id.get();
                    if val.is_empty() {
                        "home-toasts".to_string()
                    } else {
                        format!("{}-toasts", val)
                    }
                })
            />
        </main>
    }
}
