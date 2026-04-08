use super::kanji_item::KanjiItem;
use crate::i18n::{t, use_i18n};
use crate::ui_components::{Text, TextSize, TypographyVariant};
use leptos::prelude::*;
use origa::dictionary::kanji::KanjiInfo;
use std::collections::HashSet;

#[component]
pub fn KanjiList(
    kanji_list: Vec<&'static KanjiInfo>,
    selected_kanji: RwSignal<HashSet<String>>,
    known_kanji: HashSet<String>,
) -> impl IntoView {
    let i18n = use_i18n();
    if kanji_list.is_empty() {
        return view! {
            <div data-testid="kanji-drawer-empty">
                <Text size=TextSize::Small variant=TypographyVariant::Muted>
                    {t!(i18n, kanji_page.no_kanji_for_level)}
                </Text>
            </div>
        }
        .into_any();
    }

    view! {
        <div class="space-y-2 overflow-y-auto">
            <For
                each=move || kanji_list.clone()
                key=|kanji| kanji.kanji().to_string()
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
