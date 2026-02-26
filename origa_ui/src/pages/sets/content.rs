use super::actions::create_import_action;
use super::sets_level_group::SetsLevelGroup;
use super::types::{JlptLevel, SetInfo, classify_set};
use crate::repository::SupabaseUserRepository;
use crate::ui_components::{Spinner, Text, TextSize};
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::application::ListWellKnownSetsUseCase;
use origa::domain::{User, WellKnownSets};

#[component]
pub fn SetsContent() -> impl IntoView {
    let current_user = use_context::<RwSignal<Option<User>>>().expect("current_user context");
    let repository = use_context::<SupabaseUserRepository>().expect("repository context");
    let llm_service =
        use_context::<origa::infrastructure::LlmServiceInvoker>().expect("llm_service context");

    let sets: RwSignal<Vec<SetInfo>> = RwSignal::new(Vec::new());
    let importing: RwSignal<Option<WellKnownSets>> = RwSignal::new(None);
    let import_result: RwSignal<Option<String>> = RwSignal::new(None);
    let is_loading: RwSignal<bool> = RwSignal::new(true);

    let repository_for_load = repository.clone();
    let current_user_for_load = current_user;
    let sets_for_load = sets;

    spawn_local(async move {
        if let Some(user) = current_user_for_load.get_untracked() {
            let use_case = ListWellKnownSetsUseCase::new(&repository_for_load);
            if let Ok(set_infos) = use_case.execute(user.id()).await {
                let set_list: Vec<SetInfo> = set_infos
                    .into_iter()
                    .map(|info| {
                        let (set_type, level) = classify_set(&info.set);
                        let word_count = origa::domain::load_well_known_set(&info.set)
                            .map(|s| s.words().len())
                            .unwrap_or(0);
                        SetInfo {
                            set: info.set,
                            title: info.title,
                            description: info.description,
                            word_count,
                            set_type,
                            level,
                        }
                    })
                    .collect();
                sets_for_load.set(set_list);
                is_loading.set(false);
            }
        }
    });

    let on_import = create_import_action(
        repository.clone(),
        llm_service.clone(),
        current_user,
        importing,
        import_result,
    );

    view! {
        <div class="sets-page">
            <Show when=move || import_result.get().is_some()>
                <div class="mb-4 p-4 border border-[var(--border-dark)] bg-[var(--bg-paper)]">
                    <Text size=TextSize::Default>
                        {move || import_result.get().unwrap_or_default()}
                    </Text>
                </div>
            </Show>
            <Show when=move || is_loading.get()>
                <div class="flex justify-center py-8">
                    <Spinner />
                </div>
            </Show>
            <Show when=move || !is_loading.get()>
                <SetsLevelGroup level=JlptLevel::N5 sets=sets importing=importing on_import=on_import />
                <SetsLevelGroup level=JlptLevel::N4 sets=sets importing=importing on_import=on_import />
                <SetsLevelGroup level=JlptLevel::N3 sets=sets importing=importing on_import=on_import />
                <SetsLevelGroup level=JlptLevel::N2 sets=sets importing=importing on_import=on_import />
                <SetsLevelGroup level=JlptLevel::N1 sets=sets importing=importing on_import=on_import />
            </Show>
        </div>
    }
}
