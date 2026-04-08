use super::filters::{TypeFilter, available_set_types};
use super::sets_type_group::SetsTypeGroup;
use super::types::SetInfo;
use crate::i18n::use_i18n;
use crate::ui_components::{Heading, HeadingLevel};
use leptos::prelude::*;
use origa::domain::JapaneseLevel;
use std::collections::HashSet;

#[component]
pub fn SetsLevelGroup(
    level: JapaneseLevel,
    sets: Memo<Vec<SetInfo>>,
    type_filter: RwSignal<TypeFilter>,
    known_kanji: HashSet<String>,
    on_import: Callback<(String, String)>,
    selected_sets: RwSignal<HashSet<String>>,
    on_toggle_select: Callback<String>,
) -> impl IntoView {
    let i18n = use_i18n();
    let sets_for_level = Memo::new(move |_| {
        sets.get()
            .into_iter()
            .filter(|s| s.level == level)
            .collect::<Vec<_>>()
    });

    let available_types = Memo::new(move |_| {
        let sets_for_level_vec = sets_for_level.get();
        available_set_types()
            .into_iter()
            .filter(|type_meta| {
                sets_for_level_vec
                    .iter()
                    .any(|set| set.set_type == type_meta.id)
            })
            .collect::<Vec<_>>()
    });

    let known_kanji_stored = StoredValue::new(known_kanji);

    view! {
        <Show when=move || !sets_for_level.get().is_empty()>
            <div class="sets-group">
                <Heading
                    level=Signal::derive(|| HeadingLevel::H3)
                    class=Signal::derive(|| "mb-4".to_string())
                >
                    {i18n.get_keys().sets().level_label().inner().to_string().replacen("{}", level.code(), 1)}
                </Heading>
                    <For
                        each=move || available_types.get()
                        key=|type_meta| type_meta.id.clone()
                        children=move |type_meta| {
                            view! {
                                <SetsTypeGroup
                                    set_type=type_meta.id.clone()
                                    sets_for_level=sets_for_level
                                    type_filter=type_filter
                                    known_kanji=known_kanji_stored.get_value()
                                    on_import=on_import
                                    selected_sets=selected_sets
                                    on_toggle_select=on_toggle_select
                                />
                            }
                        }
                    />
            </div>
        </Show>
    }
}
