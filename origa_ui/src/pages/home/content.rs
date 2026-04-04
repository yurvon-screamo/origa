use super::{calculate_stats, format_delta, format_number, PrimaryStats, SecondaryStats};
use super::{HistoryModal, HomeSkeleton, JlptProgressCard, JlptSkeleton, StatMetric, StatsGrid};
use crate::repository::{set_last_sync_time, HybridUserRepository};
use crate::ui_components::{
    Text, TextSize, ToastContainer, ToastData, ToastType, TypographyVariant,
};
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::domain::{DailyHistoryItem, JlptProgress};
use origa::traits::UserRepository;

const SYNC_TOAST_ID: usize = usize::MAX;

#[component]
pub fn HomeContent(#[prop(optional, into)] test_id: Signal<String>) -> impl IntoView {
    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() {
            None
        } else {
            Some(val)
        }
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
    let repo_for_sync = repository.clone();
    let disposed = StoredValue::new(());

    Effect::new(move |_| {
        let repo = repo_for_sync.clone();

        toasts.update(|t| {
            t.push(ToastData {
                id: SYNC_TOAST_ID,
                toast_type: ToastType::Info,
                title: "Синхронизация".to_string(),
                message: "Синхронизация данных с сервером...".to_string(),
                duration_ms: None,
                closable: false,
            });
        });

        spawn_local(async move {
            match repo.merge_current_user().await {
                Ok(()) => {
                    if disposed.is_disposed() {
                        return;
                    }
                    toasts.update(|t| t.retain(|toast| toast.id != SYNC_TOAST_ID));
                    toasts.update(|t| {
                        t.push(ToastData {
                            id: t.len(),
                            toast_type: ToastType::Success,
                            title: "Синхронизация".to_string(),
                            message: "Данные успешно синхронизированы".to_string(),
                            duration_ms: None,
                            closable: true,
                        });
                    });
                    set_last_sync_time(js_sys::Date::now() as u64 / 1000);
                },
                Err(e) => {
                    if disposed.is_disposed() {
                        return;
                    }
                    toasts.update(|t| t.retain(|toast| toast.id != SYNC_TOAST_ID));
                    toasts.update(|t| {
                        t.push(ToastData {
                            id: t.len(),
                            toast_type: ToastType::Error,
                            title: "Ошибка синхронизации".to_string(),
                            message: e.to_string(),
                            duration_ms: None,
                            closable: true,
                        });
                    });
                },
            }
        });
    });

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

                    <div class="flex items-center gap-2"></div>
                </div>

                <Show
                    when=move || !is_loading.get()
                    fallback=move || view! { <JlptSkeleton /> }
                >
                    <JlptProgressCard
                        jlpt_progress=Signal::derive(move || jlpt_progress.get())
                        test_id=Signal::derive(move || {
                            let val = test_id.get();
                            if val.is_empty() { "home-jlpt-progress".to_string() } else { format!("{}-jlpt-progress", val) }
                        })
                    />
                </Show>

                <Show
                    when=move || !is_loading.get()
                    fallback=move || view! { <HomeSkeleton /> }
                >
                    <StatsGrid
                        total_cards=Signal::derive(move || format_number(primary.get().total_cards))
                        total_cards_delta=Signal::derive(move || format_delta(primary.get().total_cards_delta))
                        learned=Signal::derive(move || format_number(primary.get().learned))
                        learned_delta=Signal::derive(move || format_delta(primary.get().learned_delta))
                        in_progress=Signal::derive(move || format_number(primary.get().in_progress))
                        in_progress_delta=Signal::derive(move || format_delta(primary.get().in_progress_delta))
                        new_cards=Signal::derive(move || format_number(primary.get().new))
                        new_delta=Signal::derive(move || format_delta(primary.get().new_delta))
                        high_difficulty=Signal::derive(move || format_number(secondary.get().high_difficulty))
                        high_difficulty_delta=Signal::derive(move || format_delta(secondary.get().high_difficulty_delta))
                        positive=Signal::derive(move || format_number(secondary.get().positive))
                        positive_delta=Signal::derive(move || format_delta(secondary.get().positive_delta))
                        negative=Signal::derive(move || format_number(secondary.get().negative))
                        negative_delta=Signal::derive(move || format_delta(secondary.get().negative_delta))
                        total_ratings=Signal::derive(move || format_number(secondary.get().total_ratings))
                        total_ratings_delta=Signal::derive(move || format_delta(secondary.get().total_ratings_delta))
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
                    if val.is_empty() { "home-history-modal".to_string() } else { format!("{}-history-modal", val) }
                })
            />

            <ToastContainer
                toasts=toasts
                duration_ms=5000
                test_id=Signal::derive(move || {
                    let val = test_id.get();
                    if val.is_empty() { "home-toasts".to_string() } else { format!("{}-toasts", val) }
                })
            />
        </main>
    }
}
