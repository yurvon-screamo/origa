use super::actions::create_import_action;
use super::sets_level_group::SetsLevelGroup;
use super::types::{ImportResult, ImportState, SetInfo};
use crate::repository::HybridUserRepository;
use crate::ui_components::{LoadingOverlay, Spinner, ToastContainer, ToastData, ToastType};
use crate::well_known_set::WellKnownSetLoaderImpl;
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::application::ListWellKnownSetsUseCase;
use origa::domain::{JapaneseLevel, User};

static mut TOAST_COUNTER: usize = 0;

fn next_toast_id() -> usize {
    unsafe {
        TOAST_COUNTER += 1;
        TOAST_COUNTER
    }
}

#[component]
pub fn SetsContent() -> impl IntoView {
    let current_user = use_context::<RwSignal<Option<User>>>().expect("current_user context");
    let repository = use_context::<HybridUserRepository>().expect("repository context");
    let llm_service =
        use_context::<origa::infrastructure::LlmServiceInvoker>().expect("llm_service context");

    let sets: RwSignal<Vec<SetInfo>> = RwSignal::new(Vec::new());
    let importing: RwSignal<Option<ImportState>> = RwSignal::new(None);
    let toasts: RwSignal<Vec<ToastData>> = RwSignal::new(Vec::new());
    let is_loading: RwSignal<bool> = RwSignal::new(true);

    let sets_for_load = sets;
    let loader = WellKnownSetLoaderImpl::new();

    spawn_local(async move {
        let use_case = ListWellKnownSetsUseCase::new(&loader);
        if let Ok(set_infos) = use_case.execute().await {
            let set_list: Vec<SetInfo> = set_infos
                .into_iter()
                .map(|info| SetInfo {
                    set_id: info.meta.id,
                    title: info.meta.title_ru,
                    description: info.meta.desc_ru,
                    word_count: info.word_count,
                    set_type: info.meta.set_type,
                    level: info.meta.level,
                })
                .collect();
            sets_for_load.set(set_list);
            is_loading.set(false);
        }
    });

    let toasts_signal = toasts;
    let on_import = create_import_action(
        repository.clone(),
        llm_service.clone(),
        current_user,
        importing,
        Callback::new(move |result: ImportResult| {
            let toast_type = if result.is_success {
                ToastType::Success
            } else {
                ToastType::Error
            };
            let title = if result.is_success {
                "Импорт завершён"
            } else {
                "Ошибка импорта"
            };
            toasts_signal.update(|t| {
                t.push(ToastData {
                    id: next_toast_id(),
                    toast_type,
                    title: title.to_string(),
                    message: result.message,
                });
            });
        }),
    );

    view! {
        <div class="sets-page">
            <Show when=move || importing.get().is_some()>
                <LoadingOverlay
                    message=Signal::derive(move || {
                        importing.get()
                            .map(|s| format!("Импорт: {}...", s.title))
                            .unwrap_or_default()
                    })
                />
            </Show>
            <Show when=move || is_loading.get()>
                <div class="flex justify-center py-8">
                    <Spinner />
                </div>
            </Show>
            <Show when=move || !is_loading.get()>
                <SetsLevelGroup level=JapaneseLevel::N5 sets=sets importing=importing on_import=on_import />
                <SetsLevelGroup level=JapaneseLevel::N4 sets=sets importing=importing on_import=on_import />
                <SetsLevelGroup level=JapaneseLevel::N3 sets=sets importing=importing on_import=on_import />
                <SetsLevelGroup level=JapaneseLevel::N2 sets=sets importing=importing on_import=on_import />
                <SetsLevelGroup level=JapaneseLevel::N1 sets=sets importing=importing on_import=on_import />
            </Show>
            <ToastContainer toasts=toasts duration_ms=5000 />
        </div>
    }
}
