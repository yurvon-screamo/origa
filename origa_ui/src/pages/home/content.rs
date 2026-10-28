use super::AccuracyChartCard;
use super::content_sync::{
    run_sync, show_sync_error_toast, show_sync_success_toast, show_sync_toast,
};
use super::{
    CompletionForecast, JftReadinessCard, JlptProgressCard, JlptSkeleton, RecentlyStudiedItem,
    StudiedTodayList, TodayOverview, TodayOverviewCard, WelcomeCard, compute_completion_forecast,
    compute_due_count, compute_studied_today, compute_today_overview,
};
use crate::i18n::use_i18n;
use crate::loaders::JftCandoLoaderImpl;
use crate::loaders::recalculate_user_jlpt_progress;
use crate::repository::{HybridUserRepository, get_last_sync_time, set_last_sync_time};
use crate::store::ConnectivityStore;
use crate::store::lesson_handoff::{
    POLL_DEADLINE_MS, POLL_STEP_MS, clear_handoff, has_handoff, take_handoff,
};
use crate::ui_components::{ToastContainer, ToastData};
use crate::utils::display_name::display_name_for;
use chrono::Utc;
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::domain::JftCoverage;
use origa::domain::JlptProgress;
use origa::domain::insights::{AccuracySeries, compute_daily_accuracy};
use origa::domain::local_day;
use origa::traits::UserRepository;

use std::collections::HashSet;

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[test]
    fn is_sync_fresh_within_window_is_true() {
        assert!(is_sync_fresh(Some(0)));
        assert!(is_sync_fresh(Some(SYNC_FRESH_WINDOW_MS - 1)));
    }

    #[test]
    fn is_sync_fresh_at_or_past_window_is_false() {
        assert!(!is_sync_fresh(Some(SYNC_FRESH_WINDOW_MS)));
        assert!(!is_sync_fresh(Some(SYNC_FRESH_WINDOW_MS + 60_000)));
    }

    #[test]
    fn is_sync_fresh_without_a_previous_sync_is_false() {
        assert!(!is_sync_fresh(None), "no stamp must never suppress a sync");
    }

    #[test]
    fn init_paint_is_blocked_only_by_a_sync_paint() {
        assert!(init_should_apply(StatsPainter::None));
        assert!(init_should_apply(StatsPainter::Init));
        assert!(
            !init_should_apply(StatsPainter::Sync),
            "a late init read must not overwrite fresher sync-painted stats"
        );
    }

    #[rstest]
    #[case::no_imports(HashSet::new(), false)]
    #[case::jlpt_only(HashSet::from(["jlpt_n5".to_string(), "migii_n5_1".to_string()]), false)]
    #[case::irodori_present(HashSet::from(["irodori_nyuumon_01".to_string()]), true)]
    #[case::irodori_among_others(
        HashSet::from(["jlpt_n4".to_string(), "irodori_shokyuu2_18".to_string()]),
        true
    )]
    fn irodori_imports_are_detected_only_for_irodori_sets(
        #[case] imported: HashSet<String>,
        #[case] expected: bool,
    ) {
        assert_eq!(
            has_irodori_imports(&imported),
            expected,
            "imported: {imported:?}"
        );
    }
}

/// A sync that succeeded less than this window ago suppresses the home
/// mount's sync entirely (zero requests): the lesson-complete screen (or a
/// previous mount) just finished one. A local change inside the window is
/// not lost — the dirty flag defers its push to the next sync trigger
/// outside the window (next mount, connectivity flip, lesson end).
pub(crate) const SYNC_FRESH_WINDOW_MS: u64 = 30_000;

/// Whether a completed sync is recent enough to skip the home mount's own.
/// Pure decision core of the freshness gate.
fn is_sync_fresh(ms_since_last_sync: Option<u64>) -> bool {
    ms_since_last_sync.is_some_and(|ago| ago < SYNC_FRESH_WINDOW_MS)
}

/// Which component painted the home stats most recently. A sync-passed
/// snapshot is strictly fresher than an init-passed one (it is read after
/// the merge), so a slow init read finishing late must not overwrite it.
#[derive(Clone, Copy, PartialEq, Eq)]
enum StatsPainter {
    None,
    Init,
    Sync,
}

/// Init may paint only when the sync has not painted yet. `Init` re-applies
/// (impossible in practice — init runs once per mount) stays allowed.
fn init_should_apply(painted_by: StatsPainter) -> bool {
    !matches!(painted_by, StatsPainter::Sync)
}

/// Consumes the lesson-exit handoff with a bounded wait: the lesson screen's
/// parallel read usually lands within milliseconds, so the stats paint on
/// the first home render instead of after a second full read. Empty handoff
/// takes the fast path immediately (the common mount). On timeout the
/// handoff is cleared — no residue may paint stale stats on a later mount —
/// and the caller falls back to its own read.
async fn take_handoff_bounded() -> Option<origa::domain::User> {
    if !has_handoff() {
        return None;
    }
    let deadline = js_sys::Date::now() as u64 + POLL_DEADLINE_MS;
    loop {
        if let Some(user) = take_handoff() {
            return Some(user);
        }
        if js_sys::Date::now() as u64 >= deadline {
            clear_handoff();
            return None;
        }
        gloo_timers::future::TimeoutFuture::new(POLL_STEP_MS as u32).await;
    }
}

/// Whether any imported set belongs to the Irodori/JFT track. Users without
/// Irodori imports never trigger the readiness-card fetch: the card stays
/// hidden and no CDN request is made.
fn has_irodori_imports(imported_sets: &HashSet<String>) -> bool {
    imported_sets.iter().any(|id| id.starts_with("irodori_"))
}

/// Irodori learner? Fetch the Can-do catalog and compute band coverage over
/// the imported lessons. JLPT-only users never hit the network here; a
/// catalog failure just keeps the readiness card hidden.
fn spawn_jft_coverage_fetch(
    imported_sets: HashSet<String>,
    jft_coverage: RwSignal<Option<JftCoverage>>,
    disposed: StoredValue<()>,
) {
    if !has_irodori_imports(&imported_sets) {
        return;
    }
    spawn_local(async move {
        match JftCandoLoaderImpl::new().load().await {
            Ok(catalog) => {
                if disposed.is_disposed() {
                    return;
                }
                jft_coverage.set(Some(catalog.coverage(&imported_sets)));
            },
            Err(e) => {
                tracing::warn!("JFT readiness catalog unavailable, card hidden: {e}");
            },
        }
    });
}

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
    let accuracy_series = RwSignal::new(AccuracySeries { days: Vec::new() });
    let today_overview = RwSignal::new(TodayOverview::default());
    let due_count = RwSignal::new(0usize);
    let recent_studied = RwSignal::new(Vec::<RecentlyStudiedItem>::new());
    let known_kanji: RwSignal<HashSet<char>> = RwSignal::new(HashSet::new());
    let forecast: RwSignal<CompletionForecast> = RwSignal::new(CompletionForecast::default());
    // JFT-Basic readiness (#760): None hides the card — no Irodori imports,
    // or the CDN catalog failed to load (offline paths stay quiet).
    let jft_coverage: RwSignal<Option<JftCoverage>> = RwSignal::new(None);

    let is_loading = RwSignal::new(true);
    let user_name: RwSignal<String> = RwSignal::new(String::new());
    let toasts: RwSignal<Vec<ToastData>> = RwSignal::new(Vec::new());
    let stats_painter = StoredValue::new(StatsPainter::None);
    let disposed = StoredValue::new(());

    let persist_user = move |repo: HybridUserRepository, user: origa::domain::User| {
        spawn_local(async move {
            if disposed.is_disposed() {
                return;
            }
            if let Err(e) = repo.save(&user).await {
                tracing::warn!("Failed to persist JLPT recalculation: {e}");
            }
        });
    };

    let repo_for_init = repository.clone();
    Effect::new(move |_| {
        let repo = repo_for_init.clone();
        spawn_local(async move {
            // The lesson-exit handoff wins when it is ready: its snapshot was
            // read in parallel with the navigation, so the stats paint on the
            // first render. Timeout/absent → the usual local read.
            let (read, from_handoff) = match take_handoff_bounded().await {
                Some(user) => (Ok(Some(user)), true),
                None => (repo.get_current_user().await, false),
            };

            match read {
                Ok(Some(mut user)) => {
                    if disposed.is_disposed() {
                        return;
                    }
                    let progress_before_recalc = user.jlpt_progress().clone();
                    recalculate_user_jlpt_progress(&mut user);
                    user_name.set(display_name_for(user.username(), user.email()));

                    // Diagnostics for the "stats stale until sync" report:
                    // which snapshot painted, how old it is, how many cards
                    // it carried.
                    tracing::info!(
                        source = if from_handoff { "lesson_handoff" } else { "local_read" },
                        updated_at = %user.updated_at(),
                        study_cards = user.knowledge_set().study_cards().len(),
                        "Home init stats snapshot"
                    );

                    if init_should_apply(stats_painter.get_value()) {
                        let ks = user.knowledge_set();
                        jlpt_progress.set(user.jlpt_progress().clone());
                        known_kanji.set(ks.get_known_kanji());
                        accuracy_series.set(compute_daily_accuracy(
                            ks.lesson_history(),
                            local_day::local_offset(),
                            Utc::now(),
                        ));
                        today_overview.set(compute_today_overview(ks, ks.lesson_history()));
                        due_count.set(compute_due_count(ks));
                        recent_studied.set(compute_studied_today(ks, user.native_language()));
                        forecast.set(compute_completion_forecast(
                            ks,
                            ks.lesson_history(),
                            user.native_language(),
                        ));
                        stats_painter.set_value(StatsPainter::Init);
                    } else {
                        tracing::debug!("Home init skipped stats paint: sync already painted");
                    }

                    spawn_jft_coverage_fetch(user.imported_sets().clone(), jft_coverage, disposed);

                    is_loading.set(false);

                    // Persist only when the recalc actually changed the
                    // progress: an unconditional save marks the sync state
                    // dirty on every home mount and defeats the sync
                    // short-circuit (ADR-045). The check runs BEFORE any
                    // clone — a by-value `persist_user` would otherwise
                    // deep-copy the multi-megabyte user for nothing.
                    if user.jlpt_progress() != &progress_before_recalc {
                        persist_user(repo.clone(), user);
                    }
                },
                Ok(None) | Err(_) => {
                    if disposed.is_disposed() {
                        return;
                    }
                    // Diagnostics for the "stats stale until sync" report:
                    // the init read must not degrade silently — both paths
                    // leave the defaults on screen.
                    match &read {
                        Ok(None) => tracing::error!(
                            "Home init: no local user record (stats stay defaults); \
                             expecting the post-login restore or a login redirect"
                        ),
                        Err(e) => tracing::error!("Home init: local read failed: {e:?}"),
                        Ok(Some(_)) => unreachable!("matched above"),
                    }
                    is_loading.set(false);
                },
            }
        });
    });

    let repo_sync = repository.clone();
    let i18n_sync = i18n;
    let connectivity = use_context::<ConnectivityStore>();
    Effect::new(move |_| {
        let repo = repo_sync.clone();
        let i18n = i18n_sync;
        let is_online = connectivity
            .as_ref()
            .map(|c| c.is_online.get())
            .unwrap_or(true);
        if !is_online {
            return;
        }
        // Freshness gate: a sync that just succeeded (lesson-complete
        // screen, a previous mount) makes this pass redundant — zero
        // requests, zero toasts. A local change inside the window defers
        // its push to the next trigger outside it (dirty flag keeps it
        // safe). The stored stamp is in seconds (see set_last_sync_time).
        let now_ms = js_sys::Date::now() as u64;
        let ms_since_last_sync =
            get_last_sync_time().map(|t| now_ms.saturating_sub(t.saturating_mul(1000)));
        if is_sync_fresh(ms_since_last_sync) {
            tracing::debug!("Home sync skipped: a sync succeeded within the freshness window");
            return;
        }
        spawn_local(async move {
            show_sync_toast(toasts, i18n);

            let save_repo = repo.clone();
            match run_sync(repo).await {
                Ok(Some(mut user)) => {
                    if disposed.is_disposed() {
                        return;
                    }
                    // Same conditional-persist contract as the init effect:
                    // only write (and deep-copy the user) when the recalc
                    // changed the progress (ADR-045).
                    let progress_before_recalc = user.jlpt_progress().clone();
                    recalculate_user_jlpt_progress(&mut user);
                    let ks = user.knowledge_set();
                    jlpt_progress.set(user.jlpt_progress().clone());
                    known_kanji.set(ks.get_known_kanji());
                    accuracy_series.set(compute_daily_accuracy(
                        ks.lesson_history(),
                        local_day::local_offset(),
                        Utc::now(),
                    ));
                    today_overview.set(compute_today_overview(ks, ks.lesson_history()));
                    due_count.set(compute_due_count(ks));
                    recent_studied.set(compute_studied_today(ks, user.native_language()));
                    forecast.set(compute_completion_forecast(
                        ks,
                        ks.lesson_history(),
                        user.native_language(),
                    ));
                    stats_painter.set_value(StatsPainter::Sync);
                    show_sync_success_toast(toasts, i18n);
                    set_last_sync_time(js_sys::Date::now() as u64 / 1000);

                    if user.jlpt_progress() != &progress_before_recalc {
                        persist_user(save_repo, user);
                    }
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
                    show_sync_error_toast(toasts, i18n, &e);
                },
            }
        });
    });

    view! {
        <main class="flex-1" data-testid=test_id_val>
            <div class="py-4 sm:py-5 space-y-6 sm:space-y-8">
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

                    <Show when=move || jft_coverage.get().is_some()>
                        <JftReadinessCard
                            coverage=Signal::derive(move || {
                                jft_coverage.get().expect("guarded by the enclosing Show")
                            })
                            test_id=Signal::derive(|| "home-jft-readiness".to_string())
                        />
                    </Show>

                    <TodayOverviewCard
                        overview=Signal::derive(move || today_overview.get())
                        forecast=Signal::derive(move || forecast.get())
                        due_count=Signal::derive(move || due_count.get())
                        test_id=Signal::derive(|| "home-today-overview".to_string())
                    />

                    <AccuracyChartCard
                        series=accuracy_series.into()
                        test_id=Signal::derive(|| "home-accuracy-chart".to_string())
                    />

                    <StudiedTodayList
                        items=Signal::derive(move || recent_studied.get())
                        known_kanji=Signal::derive(move || known_kanji.get())
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
