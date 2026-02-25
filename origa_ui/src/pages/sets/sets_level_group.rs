use super::sets_type_group::SetsTypeGroup;
use super::types::{JlptLevel, SetInfo, SetType};
use leptos::prelude::*;
use origa::domain::WellKnownSets;

#[component]
pub fn SetsLevelGroup(
    level: JlptLevel,
    sets: RwSignal<Vec<SetInfo>>,
    importing: RwSignal<Option<WellKnownSets>>,
    on_import: Callback<WellKnownSets>,
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
                    {format!("Уровень {}", level.label())}
                </div>
                <SetsTypeGroup
                    set_type=SetType::Jlpt
                    sets_for_level=sets_for_level
                    importing=importing
                    on_import=on_import
                />
                <SetsTypeGroup
                    set_type=SetType::Migii
                    sets_for_level=sets_for_level
                    importing=importing
                    on_import=on_import
                />
            </div>
        </Show>
    }
}
