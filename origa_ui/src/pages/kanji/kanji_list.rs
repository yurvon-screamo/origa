use super::kanji_item::KanjiItem;
use crate::ui_components::{Text, TextSize, TypographyVariant};
use leptos::prelude::*;
use origa::use_cases::KanjiItemInfo;
use std::collections::HashSet;

#[component]
pub fn KanjiList(
    kanji_list: Vec<KanjiItemInfo>,
    selected_kanji: RwSignal<HashSet<String>>,
    known_kanji: HashSet<String>,
) -> impl IntoView {
    if kanji_list.is_empty() {
        return view! {
            <Text size=TextSize::Small variant=TypographyVariant::Muted>
                "Нет кандзи для выбранного уровня (или все уже изучены)"
            </Text>
        }
        .into_any();
    }

    view! {
        <div class="space-y-2 overflow-y-auto">
            <For
                each=move || kanji_list.clone()
                key=|kanji| kanji.kanji.to_string()
                children=move |kanji_info| {
                    view! {
                        <KanjiItem
                            kanji_info=kanji_info
                            selected_kanji=selected_kanji
                            known_kanji=known_kanji.clone()
                        />
                    }
                }
            />
        </div>
    }
    .into_any()
}
