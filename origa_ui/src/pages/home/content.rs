use super::{HistoryModal, HomeSkeleton, JlptProgressCard, JlptSkeleton, StatMetric, StatsGrid};
use super::{HomeStats, calculate_stats, format_delta, format_number};
use crate::repository::session;
use crate::repository::{HybridUserRepository, SyncContext};
use crate::ui_components::{Spinner, Text, TextSize, ToastContainer, ToastData, ToastType, TypographyVariant};
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::domain::{DailyHistoryItem, JlptProgress, User};
use origa::use_cases::GetUserInfoUseCase;

#[component]
pub fn HomeContent() -> impl IntoView {
    let current_user =
        use_context::<RwSignal<Option<User>>>().expect("current_user context not provided");
    let repository =
        use_context::<HybridUserRepository>().expect("repository context not provided");
    let sync_context = use_context::<SyncContext>().expect("sync_context not provided");

    let stats = RwSignal::new(None::<HomeStats>);
    let history = RwSignal::new(Vec::<DailyHistoryItem>::new());
    let history_open = RwSignal::new(false);
    let selected_metric = RwSignal::new(StatMetric::TotalCards);
    let is_loading = RwSignal::new(true);
    let is_syncing = RwSignal::new(true);
    let jlpt_progress = RwSignal::new(JlptProgress::new());
    let toasts: RwSignal<Vec<ToastData>> = RwSignal::new(Vec::new());

    let repo_for_sync = repository.clone();
    let sync_ctx_for_sync = sync_context;
    let user_for_sync = current_user;
    Effect::new(move |_| {
        let user = current_user.get();
        if let Some(user) = user {
            let user_id = user.id();
            let repo = repo_for_sync.clone();
            let sync_ctx = sync_ctx_for_sync;
            let current_user_signal = user_for_sync;
            spawn_local(async move {
                sync_ctx.start_sync();

                match repo.force_sync(user_id).await {
                    Ok(Some(merged_user)) => {
                        current_user_signal.set(Some(merged_user));
                        session::set_last_sync_time(js_sys::Date::now() as u64 / 1000);
                        tracing::info!("Home: force_sync completed");
                    }
                    Ok(None) => {
                        session::set_last_sync_time(js_sys::Date::now() as u64 / 1000);
                        tracing::debug!("Home: force_sync - no changes");
                    }
                    Err(e) => {
                        tracing::error!("Home: force_sync error: {:?}", e);
                    }
                }
                sync_ctx.complete_sync();
                is_syncing.set(false);
            });
        }
    });

    let repo_for_init = repository.clone();
    Effect::new(move |_| {
        let user = current_user.get();
        if let Some(user) = user {
            let user_id = user.id();
            let repo = repo_for_init.clone();
            spawn_local(async move {
                let use_case = GetUserInfoUseCase::new(&repo);
                match use_case.execute(user_id).await {
                    Ok(profile) => {
                        history.set(profile.lesson_history.clone());
                        stats.set(Some(calculate_stats(&profile.lesson_history)));
                        jlpt_progress.set(profile.jlpt_progress);
                        is_loading.set(false);
                    }
                    Err(_) => {
                        stats.set(Some(HomeStats::default()));
                        is_loading.set(false);
                    }
                }
            });
        }
    });

    let repo_for_reload = repository.clone();
    Effect::new(move |_| {
        sync_context.sync_trigger.get();

        let user = current_user.get();
        if let Some(user) = user {
            let user_id = user.id();
            let repo = repo_for_reload.clone();
            spawn_local(async move {
                let use_case = GetUserInfoUseCase::new(&repo);
                if let Ok(profile) = use_case.execute(user_id).await {
                    history.set(profile.lesson_history.clone());
                    stats.set(Some(calculate_stats(&profile.lesson_history)));
                    jlpt_progress.set(profile.jlpt_progress);
                }
            });
        }
    });

    let repo_for_manual_sync = repository.clone();
    let on_manual_sync = move |_| {
        repo_for_manual_sync.sync_stats(sync_context, current_user);
    };

    Effect::new(move |_| {
        if let Some(error) = sync_context.sync_error.get() {
            let toast_id = toasts.get().len();
            toasts.update(|t| {
                t.push(ToastData {
                    id: toast_id,
                    toast_type: ToastType::Error,
                    title: "Ошибка синхронизации".to_string(),
                    message: error,
                });
            });
            sync_context.sync_error.set(None);
        }
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

                    <div class="flex items-center gap-2">
                        <Show when=move || is_syncing.get()>
                            <div class="flex items-center gap-1 text-sm text-muted-foreground">
                                <Spinner size="sm" />
                                "Синхронизация..."
                            </div>
                        </Show>

                        <Show when=move || !is_syncing.get()>
                            <Show when=move || sync_context.is_syncing.get()>
                                <div class="flex items-center gap-1 text-sm text-muted-foreground">
                                    <Spinner size="sm" />
                                    "Синхронизация..."
                                </div>
                            </Show>
                        </Show>

                        <button
                            class="text-sm text-muted-foreground hover:text-foreground transition-colors disabled:opacity-50"
                            on:click=on_manual_sync
                            disabled=move || sync_context.is_syncing.get() || is_syncing.get()
                        >
                            "Обновить"
                        </button>
                    </div>
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
