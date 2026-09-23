use crate::i18n::Locale;
use crate::loaders::recalculate_user_jlpt_progress;
use crate::repository::cdn_provider;
use crate::ui_components::{ToastData, ToastType};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_i18n::I18nContext;
use leptos_router::NavigateOptions;
use origa::traits::UserRepository;
use origa::use_cases::{
    CompleteOnboardingScoringUseCase, ImportOnboardingSetsUseCase, USERNAME_MAX_CHARS,
};
use std::sync::atomic::{AtomicUsize, Ordering};

use super::onboarding_state::OnboardingState;

/// Sequence for save-error toast ids. Every push takes a fresh id: the
/// `ToastContainer` renders through `<For key=toast.id>`, so reusing an id
/// would leave the FIRST toast node (with its own auto-dismiss timer) in
/// charge — a quick retry would get the leftover window instead of a fresh
/// one. At most one error toast exists at a time regardless: the push
/// retains out every previous error toast first.
static SAVE_ERROR_TOAST_SEQ: AtomicUsize = AtomicUsize::new(0);

/// Surfaces a failed sync checkpoint (skip / import) as an error toast. The
/// save itself hard-blocked navigation, so the user stays on the step with
/// the button re-enabled — the toast explains why nothing happened and the
/// retry is a plain re-click.
fn show_save_error_toast(toasts: RwSignal<Vec<ToastData>>, i18n: I18nContext<Locale>) {
    toasts.update(|t| {
        t.retain(|toast| toast.toast_type != ToastType::Error);
        let id = SAVE_ERROR_TOAST_SEQ.fetch_add(1, Ordering::Relaxed);
        t.push(ToastData {
            id,
            toast_type: ToastType::Error,
            title: i18n
                .get_keys_untracked()
                .onboarding()
                .save_error()
                .title()
                .inner()
                .to_string(),
            message: i18n
                .get_keys_untracked()
                .onboarding()
                .save_error()
                .message()
                .inner()
                .to_string(),
            // None = inherit the ToastContainer's duration — a single source
            // of truth for how long onboarding error toasts stay visible.
            duration_ms: None,
            closable: true,
        });
    });
}

/// Persists the display name entered on the intro step when the user moves on.
///
/// Fire-and-forget by design: the name is optional personalization, so a save
/// failure is logged and the user still proceeds (the profile page can fix the
/// name later). An empty input skips the save; an unchanged name skips the
/// write (one read still goes out to compare against the current profile).
///
/// Reads-modifies-writes the *current* profile rather than replaying a
/// snapshot captured at page load: the user can flip the language on this very
/// step, and a stale `native_language`/`daily_load` would silently roll the
/// choice back on the full-record save.
pub(super) fn create_save_intro_username_callback(
    repository: crate::repository::HybridUserRepository,
    username: RwSignal<String>,
) -> Callback<()> {
    Callback::new(move |_: ()| {
        let name: String = username
            .get_untracked()
            .trim()
            .chars()
            .take(USERNAME_MAX_CHARS)
            .collect();
        if name.is_empty() {
            return;
        }

        let repo = repository.clone();
        spawn_local(async move {
            let Ok(Some(mut user)) = repo.get_current_user().await else {
                tracing::error!("Onboarding intro: get_current_user error");
                return;
            };
            if name == user.username() {
                return;
            }

            user.set_username(name);
            if let Err(e) = repo.save_sync(&user).await {
                tracing::error!("Failed to save username on onboarding intro: {:?}", e);
            }
        });
    })
}

pub(super) fn create_on_skip_callback<N>(
    repository: crate::repository::HybridUserRepository,
    state: RwSignal<OnboardingState>,
    is_skipping: RwSignal<bool>,
    toasts: RwSignal<Vec<ToastData>>,
    i18n: I18nContext<Locale>,
    disposed: StoredValue<()>,
    navigate: N,
) -> Callback<()>
where
    N: Fn(&str, NavigateOptions) + Clone + Send + Sync + 'static,
{
    Callback::new(move |_: ()| {
        let repo = repository.clone();
        let nav = navigate.clone();
        is_skipping.set(true);
        // Snapshot before the spawn: the async chain below must not read
        // page signals after its awaits — the task stays unscoped because
        // the save_sync checkpoint is user intent and has to commit even
        // if the page is disposed mid-save (pattern: save_intro_username).
        let daily_load = state.get_untracked().daily_load;

        spawn_local(async move {
            // get_current_user is a local-only read (IndexedDB); its failure
            // is a storage anomaly no toast advice can fix — log-only by
            // design. The flag reset just re-enables the button.
            let Ok(Some(mut user)) = repo.get_current_user().await else {
                tracing::error!("Onboarding skip: get_current_user error");
                if disposed.is_disposed() {
                    return;
                }
                is_skipping.set(false);
                return;
            };

            user.set_daily_load(daily_load);
            user.mark_set_as_imported(origa::domain::ONBOARDING_SKIPPED_KEY.to_string());
            recalculate_user_jlpt_progress(&mut user);

            // Hard block on remote failure: completing onboarding is a sync
            // checkpoint, so the user must not proceed to /home without a
            // canonical remote record. This differs from the import path
            // below, which logs and continues because the import itself has
            // already committed locally by the time this save runs. The
            // failure is surfaced as an error toast and the button un-sticks
            // so the user can retry with a plain re-click.
            if let Err(e) = repo.save_sync(&user).await {
                tracing::error!("Onboarding skip: save error: {:?}", e);
                if disposed.is_disposed() {
                    return;
                }
                is_skipping.set(false);
                show_save_error_toast(toasts, i18n);
                return;
            }

            if disposed.is_disposed() {
                return;
            }
            nav("/home", Default::default());
        });
    })
}

pub(super) fn create_on_start_import_callback(
    repository: crate::repository::HybridUserRepository,
    state: RwSignal<OnboardingState>,
    is_importing: RwSignal<bool>,
    toasts: RwSignal<Vec<ToastData>>,
    i18n: I18nContext<Locale>,
    disposed: StoredValue<()>,
) -> Callback<()> {
    Callback::new(move |_: ()| {
        let repo = repository.clone();
        let cdn = cdn_provider();
        let disposed = disposed;
        is_importing.set(true);
        // Snapshot before the spawn (disposed-signal fix): both branches
        // below used to read `state` after awaits — a page disposed
        // mid-import panicked on the disposed read. The task stays
        // unscoped: the save_sync checkpoints are user intent and must
        // commit regardless of the page's lifetime.
        let daily_load = state.get_untracked().daily_load;
        let target_level = state.get_untracked().target_level();

        spawn_local(async move {
            let set_ids = state.get().get_final_sets();

            // An empty selection is a valid outcome: the user chose to import
            // nothing. Persist the choices made on the earlier steps (daily
            // load, JLPT progress) through the same sync checkpoint as the
            // skip path, then advance to Scoring — with no imported sets the
            // scoring queue is empty and completes immediately, so onboarding
            // finishes from there. Network-failure branches below surface an
            // error toast (same checkpoint semantics as the skip path); the
            // button un-sticks and the user can retry. The local-only
            // get_current_user branches stay log-only: a storage anomaly has
            // no actionable advice.
            if set_ids.is_empty() {
                let Ok(Some(mut user)) = repo.get_current_user().await else {
                    tracing::error!(
                        "Onboarding empty import: get_current_user failed or no user record"
                    );
                    if disposed.is_disposed() {
                        return;
                    }
                    is_importing.set(false);
                    return;
                };

                user.set_daily_load(daily_load);
                recalculate_user_jlpt_progress(&mut user);

                // Hard block on remote failure: this save commits the daily
                // load picked on step 2 — proceeding without it would
                // silently drop the user's choice.
                if let Err(e) = repo.save_sync(&user).await {
                    tracing::error!("Onboarding empty import: save error: {:?}", e);
                    if disposed.is_disposed() {
                        return;
                    }
                    is_importing.set(false);
                    show_save_error_toast(toasts, i18n);
                    return;
                }

                if disposed.is_disposed() {
                    return;
                }
                // Logged after the checkpoint so the message states an
                // outcome, not an intent that an error branch would refute.
                tracing::info!("No sets selected for import — advancing to scoring");
                state.update(|s| {
                    s.go_to_next_step();
                });
                is_importing.set(false);
                return;
            }

            // Read the CURRENT profile, not the page-load snapshot: the
            // intro-step name save (and other onboarding writes) land after
            // this page loaded, and the import below persists the FULL
            // record — replaying a stale snapshot would roll those writes
            // back (the lost-display-name bug).
            let Ok(Some(mut user)) = repo.get_current_user().await else {
                tracing::error!("Onboarding import: get_current_user failed or no user record");
                if disposed.is_disposed() {
                    return;
                }
                is_importing.set(false);
                return;
            };

            // Set import tokenizes the word lists (#521): the tokenizer
            // left the startup overlay, so gate on its readiness here. A
            // tokenizer miss means its CDN payload never arrived — same
            // user-facing semantics as a network failure on the import
            // itself, hence the same toast.
            if let Err(e) = crate::loaders::dictionary::ensure_tokenizer_loaded().await {
                tracing::error!("tokenizer unavailable for onboarding import: {e:?}");
                if disposed.is_disposed() {
                    return;
                }
                is_importing.set(false);
                show_save_error_toast(toasts, i18n);
                return;
            }

            // recalculate_user_jlpt_progress depends on JLPT_CONTENT (UI-side
            // CDN singleton), so it cannot move into origa/. Applied here so
            // the single save_sync inside execute persists both it and the
            // imported cards together.
            user.set_daily_load(daily_load);
            recalculate_user_jlpt_progress(&mut user);

            let use_case = ImportOnboardingSetsUseCase::new(&repo, cdn);
            let result = use_case.execute(user, set_ids, target_level).await;

            if disposed.is_disposed() {
                return;
            }
            match result {
                Ok(import_result) => {
                    tracing::info!(
                        "Imported: {} vocabulary, {} kanji, {} grammar, {} duplicates skipped",
                        import_result.created_vocabulary,
                        import_result.created_kanji,
                        import_result.created_grammar,
                        import_result.skipped_duplicates
                    );

                    state.update(|s| {
                        s.go_to_next_step();
                    });
                    is_importing.set(false);
                },
                Err(e) => {
                    tracing::error!("Import failed: {:?}", e);
                    is_importing.set(false);
                    show_save_error_toast(toasts, i18n);
                },
            }
        });
    })
}

/// Atomically finishes onboarding scoring: clears the per-click "don't know"
/// records, marks the user as onboarding-completed (so `/home` no longer
/// bounces back to `/onboarding`), persists both via a single `save_sync`,
/// then seeds ready-to-learn phrase cards for the now-known vocabulary.
///
/// Phrases are a derivative payload: a failure inside
/// [`CompleteOnboardingScoringUseCase`] before the save_sync bubbles up as
/// `Err`, but a seed-step failure is logged and swallowed so the user can
/// still proceed to `/home`. The next dictionary load will re-run seeding
/// with the up-to-date known-vocabulary hash.
pub(super) fn create_on_finish_callback<N>(
    repository: crate::repository::HybridUserRepository,
    is_finishing: RwSignal<bool>,
    disposed: StoredValue<()>,
    navigate: N,
) -> Callback<()>
where
    N: Fn(&str, NavigateOptions) + Clone + Send + Sync + 'static,
{
    Callback::new(move |_: ()| {
        let repo = repository.clone();
        let nav = navigate.clone();
        is_finishing.set(true);
        spawn_local(async move {
            let use_case = CompleteOnboardingScoringUseCase::new(&repo);
            match use_case.execute().await {
                Ok(seeded) => {
                    tracing::info!(seeded_phrases = seeded, "Onboarding scoring completed");
                },
                Err(e) => {
                    tracing::warn!(error = ?e, "CompleteOnboardingScoring failed");
                },
            }
            if disposed.is_disposed() {
                return;
            }
            nav("/home", Default::default());
        });
    })
}
