use super::{HistoryModal, HomeSkeleton, JlptProgressCard, JlptSkeleton, StatMetric, StatsGrid};
use super::{HomeStats, calculate_stats, format_delta, format_number};
use crate::repository::{HybridUserRepository, set_last_sync_time};
use crate::ui_components::{
    Text, TextSize, ToastContainer, ToastData, ToastType, TypographyVariant,
};
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::domain::{DailyHistoryItem, JlptProgress};
use origa::traits::UserRepository;

const SYNC_TOAST_ID: usize = usize::MAX;

#[component]
pub fn HomeContent() -> impl IntoView {
    let repository =
        use_context::<HybridUserRepository>().expect("repository context not provided");

    let stats = RwSignal::new(None::<HomeStats>);
    let history = RwSignal::new(Vec::<DailyHistoryItem>::new());
    let history_open = RwSignal::new(false);
    let selected_metric = RwSignal::new(StatMetric::TotalCards);
    let is_loading = RwSignal::new(true);
    let jlpt_progress = RwSignal::new(JlptProgress::new());
    let toasts: RwSignal<Vec<ToastData>> = RwSignal::new(Vec::new());
    let repo_for_sync = repository.clone();

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
                }
                Err(e) => {
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
                }
            }
        });
    });

    let repo_for_init = repository.clone();
    Effect::new(move |_| {
        let repo = repo_for_init.clone();
        spawn_local(async move {
            match repo.get_current_user().await {
                Ok(Some(user)) => {
                    let history_items = user.knowledge_set().lesson_history().to_vec();
                    history.set(history_items.clone());
                    stats.set(Some(calculate_stats(&history_items)));
                    jlpt_progress.set(user.jlpt_progress().clone());
                    is_loading.set(false);
                }
                Ok(None) => {
                    tracing::warn!("Home: user not found");
                    stats.set(Some(HomeStats::default()));
                    is_loading.set(false);
                }
                Err(e) => {
                    tracing::error!("Home: get_current_user error: {:?}", e);
                    stats.set(Some(HomeStats::default()));
                    is_loading.set(false);
                }
            }
        });
    });

    let repo_for_reload = repository.clone();
    Effect::new(move |_| {
        let repo = repo_for_reload.clone();
        spawn_local(async move {
            match repo.get_current_user().await {
                Ok(Some(user)) => {
                    let history_items = user.knowledge_set().lesson_history().to_vec();
                    history.set(history_items.clone());
                    stats.set(Some(calculate_stats(&history_items)));
                    jlpt_progress.set(user.jlpt_progress().clone());
                }
                Ok(None) => {
                    tracing::warn!("Home: user not found on reload");
                }
                Err(e) => {
                    tracing::error!("Home: get_current_user error on reload: {:?}", e);
                }
            }
        });
    });

    let total_cards =
        Signal::derive(move || format_number(stats.get().map(|s| s.total_cards).unwrap_or(0)));
    let learned =
        Signal::derive(move || format_number(stats.get().map(|s| s.learned).unwrap_or(0)));
    let in_progress =
        Signal::derive(move || format_number(stats.get().map(|s| s.in_progress).unwrap_or(0)));
    let new_cards = Signal::derive(move || format_number(stats.get().map(|s| s.new).unwrap_or(0)));
    let high_difficulty =
        Signal::derive(move || format_number(stats.get().map(|s| s.high_difficulty).unwrap_or(0)));
    let positive =
        Signal::derive(move || format_number(stats.get().map(|s| s.positive).unwrap_or(0)));
    let negative =
        Signal::derive(move || format_number(stats.get().map(|s| s.negative).unwrap_or(0)));
    let total_ratings =
        Signal::derive(move || format_number(stats.get().map(|s| s.total_ratings).unwrap_or(0)));

    let total_cards_delta =
        Signal::derive(move || format_delta(stats.get().map(|s| s.total_cards_delta).unwrap_or(0)));
    let learned_delta =
        Signal::derive(move || format_delta(stats.get().map(|s| s.learned_delta).unwrap_or(0)));
    let in_progress_delta =
        Signal::derive(move || format_delta(stats.get().map(|s| s.in_progress_delta).unwrap_or(0)));
    let new_delta =
        Signal::derive(move || format_delta(stats.get().map(|s| s.new_delta).unwrap_or(0)));
    let high_difficulty_delta = Signal::derive(move || {
        format_delta(stats.get().map(|s| s.high_difficulty_delta).unwrap_or(0))
    });
    let positive_delta =
        Signal::derive(move || format_delta(stats.get().map(|s| s.positive_delta).unwrap_or(0)));
    let negative_delta =
        Signal::derive(move || format_delta(stats.get().map(|s| s.negative_delta).unwrap_or(0)));
    let total_ratings_delta = Signal::derive(move || {
        format_delta(stats.get().map(|s| s.total_ratings_delta).unwrap_or(0))
    });

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
        <main class="flex-1">
            <div class="w-full px-4 sm:px-6 lg:px-8 py-12">
                <div class="flex items-center justify-between mb-6">
                    <Text size=TextSize::Small variant=TypographyVariant::Muted uppercase=true tracking_widest=true>
                        "Статистика"
                    </Text>

                    <div class="flex items-center gap-2"></div>
                </div>

                <Show
                    when=move || !is_loading.get()
                    fallback=move || view! { <JlptSkeleton /> }
                >
                    <JlptProgressCard jlpt_progress=Signal::derive(move || jlpt_progress.get()) />
                </Show>

                <Show
                    when=move || !is_loading.get()
                    fallback=move || view! { <HomeSkeleton /> }
                >
                    <StatsGrid
                        total_cards=total_cards
                        total_cards_delta=total_cards_delta
                        learned=learned
                        learned_delta=learned_delta
                        in_progress=in_progress
                        in_progress_delta=in_progress_delta
                        new_cards=new_cards
                        new_delta=new_delta
                        high_difficulty=high_difficulty
                        high_difficulty_delta=high_difficulty_delta
                        positive=positive
                        positive_delta=positive_delta
                        negative=negative
                        negative_delta=negative_delta
                        total_ratings=total_ratings
                        total_ratings_delta=total_ratings_delta
                        open_history=open_history
                    />
                </Show>
            </div>

            <HistoryModal
                is_open=Signal::derive(move || history_open.get())
                metric=Signal::derive(move || selected_metric.get())
                history=Signal::derive(move || history.get())
                on_close=close_history
            />

            <ToastContainer toasts=toasts duration_ms=5000 />
        </main>
    }
}
