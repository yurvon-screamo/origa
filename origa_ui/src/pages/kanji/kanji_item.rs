use crate::ui_components::{MarkdownText, Text, TextSize, TypographyVariant};
use leptos::prelude::*;
use origa::domain::User;
use origa::use_cases::KanjiItemInfo;
use std::collections::HashSet;

#[component]
pub fn KanjiItem(
    kanji_info: KanjiItemInfo,
    selected_kanji: RwSignal<HashSet<String>>,
) -> impl IntoView {
    let current_user = use_context::<RwSignal<Option<User>>>().expect("current_user context");

    let known_kanji = Memo::new(move |_| {
        current_user
            .get()
            .map(|u| u.knowledge_set().get_known_kanji())
            .unwrap_or_default()
    });

    let kanji_str = kanji_info.kanji.to_string();
    let kanji_str_for_click = kanji_str.clone();
    let kanji_str_for_memo = kanji_str.clone();

    let is_selected = Memo::new(move |_| selected_kanji.get().contains(&kanji_str_for_memo));

    let radicals_str = kanji_info.radicals.iter().collect::<String>();
    let description = kanji_info.description.clone();

    view! {
        <div
            class=Signal::derive(move || {
                format!(
                    "p-3 border cursor-pointer transition-all {}",
                    if is_selected.get() { "border-[var(--accent-olive)] bg-[var(--bg-aged)]" } else { "border-[var(--border-dark)] bg-[var(--bg-paper)]" }
                )
            })
            on:click={
                move |_| {
                    let kanji = kanji_str_for_click.clone();
                    selected_kanji.update(|set| {
                        if set.contains(&kanji) {
                            set.remove(&kanji);
                        } else {
                            set.insert(kanji);
                        }
                    });
                }
            }
        >
            <div class="flex items-center gap-3">
                <span class="text-2xl font-serif">{kanji_info.kanji}</span>
                <div class="flex-1">
                    <MarkdownText content=Signal::derive(move || description.clone()) known_kanji=known_kanji.get()/>
                    {move || {
                        if !radicals_str.is_empty() {
                            view! {
                                <Text size=TextSize::Small variant=TypographyVariant::Muted>
                                    {format!("Радикалы: {}", radicals_str)}
                                </Text>
                            }.into_any()
                        } else {
                            ().into_any()
                        }
                    }}
                </div>
            </div>
        </div>
    }
}
