use super::filters::TypeFilter;
use super::sets_type_group::SetsTypeGroup;
use super::types::SetInfo;
use leptos::prelude::*;
use origa::domain::JapaneseLevel;
use origa::traits::SetType;

#[component]
pub fn SetsLevelGroup(
    level: JapaneseLevel,
    sets: Memo<Vec<SetInfo>>,
    type_filter: RwSignal<TypeFilter>,
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
                    type_filter=type_filter
                    on_import=on_import
                />
                <SetsTypeGroup
                    set_type=SetType::Migii
                    sets_for_level=sets_for_level
                    type_filter=type_filter
                    on_import=on_import
                />
                <SetsTypeGroup
                    set_type=SetType::SpyFamily
                    sets_for_level=sets_for_level
                    type_filter=type_filter
                    on_import=on_import
                />
                <SetsTypeGroup
                    set_type=SetType::Duolingo
                    sets_for_level=sets_for_level
                    type_filter=type_filter
                    on_import=on_import
                />
            </div>
        </Show>
    }
}
