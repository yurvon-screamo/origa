use super::content_sync::{
    run_sync, show_sync_error_toast, show_sync_success_toast, show_sync_toast,
};
use super::{
    ActivityChart, ActivityDataPoint, JlptProgressCard, JlptSkeleton, RecentStudyList,
    RecentlyStudiedItem, TodayOverview, TodayOverviewCard, WelcomeCard, compute_30day_chart_data,
    compute_recent_studied, compute_today_overview,
};
use crate::i18n::use_i18n;
use crate::repository::{HybridUserRepository, set_last_sync_time};
use crate::ui_components::{ToastContainer, ToastData};
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::domain::JlptProgress;
use origa::traits::UserRepository;

#[component]
pub fn HomeContent(#[prop(optional, into)] test_id: Signal<String>) -> impl IntoView {
    let i18n = use_i18n();
    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    let repository =
        use_context::<HybridUserRepository>().expect("repository context not provided");

    let jlpt_progress = RwSignal::new(JlptProgress::new());
    let today_overview = RwSignal::new(TodayOverview::default());
    let recent_studied = RwSignal::new(Vec::<RecentlyStudiedItem>::new());
    let chart_data = RwSignal::new(Vec::<ActivityDataPoint>::new());

    let is_loading = RwSignal::new(true);
    let user_name: RwSignal<String> = RwSignal::new(String::new());
    let toasts: RwSignal<Vec<ToastData>> = RwSignal::new(Vec::new());
    let disposed = StoredValue::new(());

    let repo_for_init = repository.clone();
    Effect::new(move |_| {
        let repo = repo_for_init.clone();
        spawn_local(async move {
            match repo.get_current_user().await {
                Ok(Some(user)) => {
                    if disposed.is_disposed() {
                        return;
                    }
                    user_name.set(user.username().to_string());

                    let ks = user.knowledge_set();
                    jlpt_progress.set(user.jlpt_progress().clone());

                    today_overview.set(compute_today_overview(ks));
                    recent_studied.set(compute_recent_studied(ks, user.native_language(), 10));
                    chart_data.set(compute_30day_chart_data(
                        ks.lesson_history(),
                        user.native_language(),
                    ));

                    is_loading.set(false);
                },
                Ok(None) => {
                    if disposed.is_disposed() {
                        return;
                    }
                    is_loading.set(false);
                },
                Err(e) => {
                    if disposed.is_disposed() {
                        return;
                    }
                    tracing::error!("Home: get_current_user error: {:?}", e);
                    is_loading.set(false);
                },
            }
        });
    });

    let repo_sync = repository.clone();
    let i18n_sync = i18n;
    Effect::new(move |_| {
        let repo = repo_sync.clone();
        let i18n = i18n_sync;
        spawn_local(async move {
            show_sync_toast(toasts, i18n);

            match run_sync(repo).await {
                Ok(Some(user)) => {
                    if disposed.is_disposed() {
                        return;
                    }
                    let ks = user.knowledge_set();
                    jlpt_progress.set(user.jlpt_progress().clone());
                    today_overview.set(compute_today_overview(ks));
                    recent_studied.set(compute_recent_studied(ks, user.native_language(), 10));
                    chart_data.set(compute_30day_chart_data(
                        ks.lesson_history(),
                        user.native_language(),
                    ));
                    show_sync_success_toast(toasts, i18n);
                    set_last_sync_time(js_sys::Date::now() as u64 / 1000);
                },
                Ok(None) => {
                    if disposed.is_disposed() {
                        return;
                    }
                    show_sync_success_toast(toasts, i18n);
                    set_last_sync_time(js_sys::Date::now() as u64 / 1000);
                },
                Err(e) => {
                    if disposed.is_disposed() {
                        return;
                    }
                    tracing::error!("Home: sync error: {:?}", e);
                    show_sync_error_toast(toasts, i18n, &e);
                },
            }
        });
    });

    view! {
        <main class="flex-1" data-testid=test_id_val>
            <div class="py-6 sm:py-8 space-y-6 sm:space-y-8">
                <WelcomeCard
                    username=Signal::from(user_name)
                    test_id=Signal::derive(|| "home-welcome".to_string())
                />

                <Show
                    when=move || !is_loading.get()
                    fallback=move || view! { <JlptSkeleton /> }
                >
                    <JlptProgressCard
                        jlpt_progress=Signal::derive(move || jlpt_progress.get())
                        test_id=Signal::derive(|| "home-jlpt-progress".to_string())
                    />

                    <div class="grid grid-cols-1 gap-6 lg:grid-cols-[minmax(280px,5fr)_minmax(360px,7fr)] lg:gap-8">
                        <TodayOverviewCard
                            overview=Signal::derive(move || today_overview.get())
                            test_id=Signal::derive(|| "home-today-overview".to_string())
                        />
                        <ActivityChart
                            chart_data=Signal::derive(move || chart_data.get())
                            test_id=Signal::derive(|| "home-activity-chart".to_string())
                        />
                    </div>

                    <RecentStudyList
                        items=Signal::derive(move || recent_studied.get())
                        test_id=Signal::derive(|| "home-recent-study".to_string())
                    />
                </Show>
            </div>

            <ToastContainer
                toasts=toasts
                duration_ms=5000
                test_id=Signal::derive(|| "home-toasts".to_string())
            />
        </main>
    }
}
