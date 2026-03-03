use super::import_set_preview_modal::ImportSetPreviewModal;
use super::sets_level_group::SetsLevelGroup;
use super::types::SetInfo;
use crate::ui_components::Spinner;
use crate::well_known_set::WellKnownSetLoaderImpl;
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::application::ListWellKnownSetsUseCase;
use origa::domain::JapaneseLevel;

#[component]
pub fn SetsContent() -> impl IntoView {
    let sets: RwSignal<Vec<SetInfo>> = RwSignal::new(Vec::new());
    let is_loading: RwSignal<bool> = RwSignal::new(true);
    let preview_modal_open = RwSignal::new(false);
    let preview_set_id = RwSignal::new(String::new());
    let preview_set_title = RwSignal::new(String::new());

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

    let on_import = Callback::new(move |(set_id, title): (String, String)| {
        preview_set_id.set(set_id);
        preview_set_title.set(title);
        preview_modal_open.set(true);
    });

    view! {
        <div class="sets-page">
            <Show when=move || is_loading.get()>
                <div class="flex justify-center py-8">
                    <Spinner />
                </div>
            </Show>
            <Show when=move || !is_loading.get()>
                <SetsLevelGroup level=JapaneseLevel::N5 sets=sets on_import=on_import />
                <SetsLevelGroup level=JapaneseLevel::N4 sets=sets on_import=on_import />
                <SetsLevelGroup level=JapaneseLevel::N3 sets=sets on_import=on_import />
                <SetsLevelGroup level=JapaneseLevel::N2 sets=sets on_import=on_import />
                <SetsLevelGroup level=JapaneseLevel::N1 sets=sets on_import=on_import />
            </Show>
            <ImportSetPreviewModal
                is_open=preview_modal_open
                set_id=Signal::derive(move || preview_set_id.get())
                set_title=Signal::derive(move || preview_set_title.get())
                on_import_result=Callback::new(move |_| {})
            />
        </div>
    }
}
