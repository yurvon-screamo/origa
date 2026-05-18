use super::dashboard_stats::RecentlyStudiedItem;
use crate::i18n::{t, use_i18n};
use crate::ui_components::{
    MarkdownText, MarkdownVariant, Tag, TagVariant, Text, TextSize, TypographyVariant,
};
use leptos::prelude::*;
use std::collections::HashSet;

fn card_type_tag(card_type: &str) -> (TagVariant, &'static str) {
    match card_type {
        "kanji" => (TagVariant::Olive, "KANJI"),
        "vocabulary" => (TagVariant::Sage, "WORDS"),
        "grammar" => (TagVariant::Terracotta, "GRAMMAR"),
        _ => (TagVariant::Olive, "KANJI"),
    }
}

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
            <div class="flex items-center gap-2 mb-1">
                <Text
                    size=TextSize::Small
                    variant=TypographyVariant::Muted
                    uppercase=true
                    tracking_widest=true
                >
                    {t!(i18n, home.studied_today)}
                </Text>
                <Text
                    size=TextSize::Small
                    variant=TypographyVariant::Muted
                    uppercase=true
                    tracking_widest=true
                >
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
                <ul class="study-list-container mt-3" style="list-style: none; padding: 0; margin: 0;">
                    <For
                        each=move || items.get()
                        key=|item| item.card_id.clone()
                        children=move |item| {
                            let (tag_variant, tag_label) = card_type_tag(&item.card_type);
                            let is_furigana = item.card_type != "grammar";
                            view! {
                                <li class="study-list-item">
                                    <div class="flex items-start gap-3 py-3">
                                        <Tag
                                            variant=Signal::derive(move || tag_variant)
                                            test_id=Signal::derive(String::new)
                                        >
                                            {tag_label}
                                        </Tag>
                                        <div class="flex-1 min-w-0">
                                            <MarkdownText
                                                content=Signal::derive(move || item.japanese.clone())
                                                known_kanji=known_kanji.get()
                                                furigana=is_furigana
                                                variant=Signal::derive(|| MarkdownVariant::Compact)
                                            test_id=Signal::derive(String::new)
                                            />
                                            <div class="font-mono text-[11px] text-[var(--fg-muted)] mt-1">
                                                {item.meaning.clone()}
                                            </div>
                                        </div>
                                    </div>
                                </li>
                            }
                        }
                    />
                </ul>
            </Show>
        </div>
    }
}
