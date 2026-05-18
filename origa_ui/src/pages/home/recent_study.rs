use super::dashboard_stats::RecentlyStudiedItem;
use crate::i18n::{t, td_string, use_i18n};
use crate::ui_components::{
    Card, MarkdownText, MarkdownVariant, Tag, TagVariant, Text, TextSize, TypographyVariant,
};
use leptos::prelude::*;
use std::collections::HashSet;

#[component]
pub fn StudiedTodayList(
    items: Signal<Vec<RecentlyStudiedItem>>,
    known_kanji: Signal<HashSet<char>>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let i18n = use_i18n();

    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    let is_empty = Signal::derive(move || items.get().is_empty());
    let count = Signal::derive(move || items.get().len());

    view! {
        <div data-testid=test_id_val>
            <div class="flex items-center gap-2 mb-4">
                <Text size=TextSize::Small variant=TypographyVariant::Muted uppercase=true tracking_widest=true>
                    {t!(i18n, home.studied_today)}
                </Text>
                <Text size=TextSize::Small variant=TypographyVariant::Muted uppercase=true tracking_widest=true>
                    {move || count.get()}
                </Text>
            </div>

            <Show when=move || is_empty.get()>
                <div class="mt-3">
                    <Text size=TextSize::Small variant=TypographyVariant::Muted>
                        {t!(i18n, home.no_studied_today)}
                    </Text>
                </div>
            </Show>

            <Show when=move || !is_empty.get()>
                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 2xl:grid-cols-5 gap-4 items-stretch">
                    <For
                        each=move || items.get()
                        key=|item| item.card_id.clone()
                        children=move |item| {
                            let card_type = item.card_type.clone();
                            let japanese = item.japanese.clone();
                            let meaning = item.meaning.clone();

                            let tag_variant = match card_type.as_str() {
                                "kanji" => TagVariant::Olive,
                                "vocabulary" => TagVariant::Sage,
                                "grammar" => TagVariant::Terracotta,
                                _ => TagVariant::Olive,
                            };

                            let is_furigana = card_type.as_str() != "grammar";
                            let is_not_grammar = card_type.as_str() != "grammar";

                            let card_type_for_label = card_type.clone();
                            let tag_label = Signal::derive(move || {
                                let locale = i18n.get_locale();
                                match card_type_for_label.as_str() {
                                    "kanji" => td_string!(locale, home.badge_kanji).to_string(),
                                    "vocabulary" => td_string!(locale, home.badge_words).to_string(),
                                    "grammar" => td_string!(locale, home.badge_grammar).to_string(),
                                    _ => td_string!(locale, home.badge_kanji).to_string(),
                                }
                            });

                            // Create Signals BEFORE view! to avoid FnOnce issues
                            let japanese_signal: Signal<String> = Signal::derive(move || japanese.clone());
                            let meaning_signal: Signal<String> = Signal::derive(move || meaning.clone());
                            let known_kanji_val = known_kanji.get();

                            view! {
                                <Card class=Signal::derive(|| "p-4 h-full flex flex-col".to_string()) test_id=Signal::derive(String::new)>
                                    <div class="flex items-center gap-2 mb-2">
                                        <Tag variant=Signal::derive(move || tag_variant) class=Signal::derive(|| "text-[10px]".to_string()) test_id=Signal::derive(String::new)>
                                            {move || tag_label.get()}
                                        </Tag>
                                        <div class="flex-1 min-w-0">
                                            <MarkdownText
                                                content=japanese_signal
                                                known_kanji=known_kanji_val
                                                furigana=is_furigana
                                                variant=Signal::derive(|| MarkdownVariant::Default)
                                                test_id=Signal::derive(String::new)
                                            />
                                        </div>
                                    </div>
                                    <Show when=move || is_not_grammar>
                                        <div class="mt-auto pt-2 line-clamp-3">
                                            <MarkdownText
                                                content=meaning_signal
                                                known_kanji=HashSet::new()
                                                furigana=false
                                                variant=Signal::derive(|| MarkdownVariant::Compact)
                                                test_id=Signal::derive(String::new)
                                            />
                                        </div>
                                    </Show>
                                </Card>
                            }
                        }
                    />
                </div>
            </Show>
        </div>
    }
}
