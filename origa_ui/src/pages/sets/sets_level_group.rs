use super::filters::TypeFilter;
use super::sets_type_group::SetsTypeGroup;
use super::types::SetInfo;
use crate::ui_components::{Heading, HeadingLevel};
use leptos::prelude::*;
use origa::domain::JapaneseLevel;
use origa::traits::SetType;
use std::collections::HashSet;

#[component]
pub fn SetsLevelGroup(
    level: JapaneseLevel,
    sets: Memo<Vec<SetInfo>>,
    type_filter: RwSignal<TypeFilter>,
    known_kanji: HashSet<String>,
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
                <Heading
                    level=Signal::derive(|| HeadingLevel::H3)
                    class=Signal::derive(|| "mb-4".to_string())
                >
                    {format!("Уровень {}", level.code())}
                </Heading>
                <SetsTypeGroup
                    set_type=SetType::Jlpt
                    sets_for_level=sets_for_level
                    type_filter=type_filter
                    known_kanji=known_kanji.clone()
                    on_import=on_import
                />
                <SetsTypeGroup
                    set_type=SetType::Migii
                    sets_for_level=sets_for_level
                    type_filter=type_filter
                    known_kanji=known_kanji.clone()
                    on_import=on_import
                />
                <SetsTypeGroup
                    set_type=SetType::SpyFamily
                    sets_for_level=sets_for_level
                    type_filter=type_filter
                    known_kanji=known_kanji.clone()
                    on_import=on_import
                />
                <SetsTypeGroup
                    set_type=SetType::DuolingoRu
                    sets_for_level=sets_for_level
                    type_filter=type_filter
                    known_kanji=known_kanji.clone()
                    on_import=on_import
                />
                <SetsTypeGroup
                    set_type=SetType::DuolingoEn
                    sets_for_level=sets_for_level
                    type_filter=type_filter
                    known_kanji=known_kanji.clone()
                    on_import=on_import
                />
            </div>
        </Show>
    }
}
