use crate::repository::cdn_provider;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::NavigateOptions;
use origa::domain::User;
use origa::traits::UserRepository;
use origa::use_cases::ImportOnboardingSetsUseCase;

use super::onboarding_state::OnboardingState;

pub(super) fn create_on_skip_callback<N>(
    repository: crate::repository::HybridUserRepository,
    state: RwSignal<OnboardingState>,
    disposed: StoredValue<()>,
    navigate: N,
) -> Callback<()>
where
    N: Fn(&str, NavigateOptions) + Clone + Send + Sync + 'static,
{
    Callback::new(move |_: ()| {
        let repo = repository.clone();
        let nav = navigate.clone();

        spawn_local(async move {
            let Ok(Some(mut user)) = repo.get_current_user().await else {
                tracing::error!("Onboarding skip: get_current_user error");
                return;
            };

            user.set_daily_load(state.get_untracked().daily_load);
            user.mark_set_as_imported("__onboarding_skipped__".to_string());

            if let Err(e) = repo.save_sync(&user).await {
                tracing::error!("Onboarding skip: save error: {:?}", e);
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
    current_user: RwSignal<Option<User>>,
    is_importing: RwSignal<bool>,
    disposed: StoredValue<()>,
) -> Callback<()> {
    Callback::new(move |_: ()| {
        let repo = repository.clone();
        let cdn = cdn_provider();
        let disposed = disposed;
        is_importing.set(true);

        spawn_local(async move {
            let set_ids = state.get().get_final_sets();

            if set_ids.is_empty() {
                tracing::warn!("No sets selected for import");
                is_importing.set(false);
                return;
            }

            let Some(user) = current_user.get() else {
                tracing::error!("User not loaded");
                is_importing.set(false);
                return;
            };

            let use_case = ImportOnboardingSetsUseCase::new(&repo, cdn);
            let result = use_case.execute(user.id(), set_ids).await;

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
                    is_importing.set(false);

                    if let Ok(Some(mut user)) = repo.get_current_user().await {
                        user.set_daily_load(state.get_untracked().daily_load);
                        if let Err(e) = repo.save_sync(&user).await {
                            tracing::error!("Failed to save daily_load: {:?}", e);
                        }
                    }

                    if disposed.is_disposed() {
                        return;
                    }

                    state.update(|s| {
                        s.go_to_next_step();
                    });
                },
                Err(e) => {
                    tracing::error!("Import failed: {:?}", e);
                    is_importing.set(false);
                },
            }
        });
    })
}
