use crate::i18n::use_i18n;
use crate::ui_components::{MarkdownText, Text, TextSize, TypographyVariant};
use leptos::prelude::*;
use origa::dictionary::kanji::KanjiInfo;
use std::collections::HashSet;

#[component]
pub fn KanjiItem(
    kanji_info: &'static KanjiInfo,
    selected_kanji: RwSignal<HashSet<String>>,
    known_kanji: HashSet<String>,
) -> impl IntoView {
    let i18n = use_i18n();
    let kanji_str = kanji_info.kanji().to_string();
    let kanji_str_for_click = kanji_str.clone();
    let kanji_str_for_memo = kanji_str.clone();

    let is_selected = Memo::new(move |_| selected_kanji.get().contains(&kanji_str_for_memo));

    let radicals_str = kanji_info.radicals_chars().iter().collect::<String>();
    let description = kanji_info.description().to_string();

    view! {
        <div
            class=Signal::derive(move || {
                format!(
                    "p-3 border cursor-pointer transition-all {}",
                    if is_selected.get() { "border-[var(--accent-olive)] bg-[var(--bg-aged)]" } else { "border-[var(--border-dark)] bg-[var(--bg-paper)]" }
                )
            })
            data-testid="kanji-drawer-item"
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
                <span class="text-2xl font-serif">{kanji_info.kanji()}</span>
                <div class="flex-1">
                    <MarkdownText content=Signal::derive(move || description.clone()) known_kanji=known_kanji.clone()/>
                    {move || {
                        let radicals_str = radicals_str.clone();
                        if !radicals_str.is_empty() {
                            view! {
                                <Text size=TextSize::Small variant=TypographyVariant::Muted>
                                    {i18n.get_keys().shared().radicals_label().inner().to_string().replacen("{}", &radicals_str, 1)}
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
