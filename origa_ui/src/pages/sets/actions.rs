use crate::app::update_current_user;
use crate::repository::HybridUserRepository;
use crate::well_known_set::WellKnownSetLoaderImpl;
use leptos::prelude::*;
use leptos::task::spawn_local;
use log::{error, info};
use origa::application::ImportWellKnownSetUseCase;
use origa::domain::User;

use super::types::{ImportResult, ImportState};

pub fn create_import_action(
    repository: HybridUserRepository,
    llm_service: origa::infrastructure::LlmServiceInvoker,
    current_user: RwSignal<Option<User>>,
    importing: RwSignal<Option<ImportState>>,
    on_result: Callback<ImportResult>,
) -> Callback<(String, String)> {
    Callback::new(move |(set_id, title): (String, String)| {
        let repo = repository.clone();
        let llm = llm_service.clone();
        let current_user = current_user;
        let importing = importing;
        let on_result = on_result;
        let title_for_state = title.clone();
        spawn_local(async move {
            if let Some(user) = current_user.get_untracked() {
                importing.set(Some(ImportState {
                    set_id: set_id.clone(),
                    title: title_for_state,
                }));
                let loader = WellKnownSetLoaderImpl::new();
                let use_case = ImportWellKnownSetUseCase::new(&repo, &llm, &loader);
                match use_case.execute(user.id(), set_id).await {
                    Ok(result) => {
                        info!(
                            "Import completed: created={}, skipped={}",
                            result.total_created_count,
                            result.skipped_words.len()
                        );
                        on_result.run(ImportResult {
                            is_success: true,
                            message: format!("Импортировано {} слов", result.total_created_count),
                        });
                        update_current_user(repo.clone(), current_user);
                    }
                    Err(e) => {
                        error!("Import failed: {}", e);
                        on_result.run(ImportResult {
                            is_success: false,
                            message: format!("{}", e),
                        });
                    }
                }
                importing.set(None);
            }
        });
    })
}
