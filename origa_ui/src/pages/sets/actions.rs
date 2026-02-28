use crate::app::update_current_user;
use crate::repository::HybridUserRepository;
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::application::ImportWellKnownSetUseCase;
use origa::domain::{User, WellKnownSets};

use super::types::{ImportResult, ImportState};

pub fn create_import_action(
    repository: HybridUserRepository,
    llm_service: origa::infrastructure::LlmServiceInvoker,
    current_user: RwSignal<Option<User>>,
    importing: RwSignal<Option<ImportState>>,
    import_result: RwSignal<Option<ImportResult>>,
) -> Callback<(WellKnownSets, String)> {
    Callback::new(move |(set, title): (WellKnownSets, String)| {
        let repo = repository.clone();
        let llm = llm_service.clone();
        let current_user = current_user;
        let importing = importing;
        let import_result = import_result;
        let title_for_state = title.clone();
        spawn_local(async move {
            if let Some(user) = current_user.get_untracked() {
                importing.set(Some(ImportState { set, title: title_for_state }));
                import_result.set(None);
                let use_case = ImportWellKnownSetUseCase::new(&repo, &llm);
                match use_case.execute(user.id(), set).await {
                    Ok(result) => {
                        import_result.set(Some(ImportResult {
                            is_success: true,
                            message: format!("Импортировано {} слов", result.total_created_count),
                        }));
                        update_current_user(repo.clone(), current_user);
                    }
                    Err(e) => {
                        import_result.set(Some(ImportResult {
                            is_success: false,
                            message: format!("{}", e),
                        }));
                    }
                }
                importing.set(None);
            }
        });
    })
}
