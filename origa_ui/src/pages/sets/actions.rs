use crate::app::update_current_user;
use crate::repository::HybridUserRepository;
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::application::ImportWellKnownSetUseCase;
use origa::domain::{User, WellKnownSets};

pub fn create_import_action(
    repository: HybridUserRepository,
    llm_service: origa::infrastructure::LlmServiceInvoker,
    current_user: RwSignal<Option<User>>,
    importing: RwSignal<Option<WellKnownSets>>,
    import_result: RwSignal<Option<String>>,
) -> Callback<WellKnownSets> {
    Callback::new(move |set: WellKnownSets| {
        let repo = repository.clone();
        let llm = llm_service.clone();
        let current_user = current_user;
        let importing = importing;
        let import_result = import_result;
        spawn_local(async move {
            if let Some(user) = current_user.get_untracked() {
                importing.set(Some(set));
                import_result.set(None);
                let use_case = ImportWellKnownSetUseCase::new(&repo, &llm);
                match use_case.execute(user.id(), set).await {
                    Ok(result) => {
                        import_result.set(Some(format!(
                            "Импортировано {} слов",
                            result.total_created_count
                        )));
                        update_current_user(repo.clone(), current_user);
                    }
                    Err(e) => {
                        import_result.set(Some(format!("Ошибка: {}", e)));
                    }
                }
                importing.set(None);
            }
        });
    })
}
