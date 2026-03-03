use super::sets_type_group::SetsTypeGroup;
use super::types::SetInfo;
use leptos::prelude::*;
use origa::application::SetType;
use origa::domain::JapaneseLevel;

#[component]
pub fn SetsLevelGroup(
    level: JapaneseLevel,
    sets: RwSignal<Vec<SetInfo>>,
    on_import: Callback<(String, String)>,
) -> impl IntoView {
    let sets_for_level = Memo::new(move |_| {
        sets.get()
            .into_iter()
            .filter(|s| s.level == level)
            .collect::<Vec<_>>()
    });

    view! {
        <Show when=move || !sets_for_level.get().is_empty()>
            <div class="sets-group">
                <div class="sets-group-title">
                    {format!("Уровень {}", level.code())}
                </div>
                <SetsTypeGroup
                    set_type=SetType::Jlpt
                    sets_for_level=sets_for_level
                    on_import=on_import
                />
                <SetsTypeGroup
                    set_type=SetType::Migii
                    sets_for_level=sets_for_level
                    on_import=on_import
                />
                <SetsTypeGroup
                    set_type=SetType::SpyFamily
                    sets_for_level=sets_for_level
                    on_import=on_import
                />
            </div>
        </Show>
    }
}
