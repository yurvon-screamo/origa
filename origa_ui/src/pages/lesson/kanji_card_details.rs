use crate::ui_components::{
    FuriganaText, KanjiViewMode, KanjiWritingSection, MarkdownText, MarkdownVariant, ReadingGroup,
    Text, TextSize, TypographyVariant,
};
use leptos::prelude::*;
use std::collections::HashSet;

#[derive(Clone, Debug, PartialEq)]
pub struct RadicalDisplay {
    pub symbol: char,
    pub name: String,
    pub description: String,
}

#[component]
pub fn KanjiCardDetails(
    kanji: String,
    radicals: Option<Vec<RadicalDisplay>>,
    example_words: Option<Vec<(String, String)>>,
    show_details: bool,
    on_readings: Option<Vec<String>>,
    kun_readings: Option<Vec<String>>,
    #[prop(into)] known_kanji: Signal<HashSet<String>>,
) -> impl IntoView {
    let kanji_stored = StoredValue::new(kanji);
    let radicals_stored = StoredValue::new(radicals);
    let examples_stored = StoredValue::new(example_words);
    let on_readings_stored = StoredValue::new(on_readings);
    let kun_readings_stored = StoredValue::new(kun_readings);

    view! {
        <Show when=move || show_details>
            <div class="my-6 space-y-4 max-w-max mx-auto">
                <ReadingGroup label="音読み[онъёми]" readings=on_readings_stored />
                <ReadingGroup label="訓読み[кунъёми]" readings=kun_readings_stored />

                <Show when=move || radicals_stored.get_value().is_some()>
                    <div class="flex gap-4 items-start text-left">
                        <div class="w-16 shrink-0">
                            <Text size=TextSize::Default variant=TypographyVariant::Muted>
                                "Радикалы"
                            </Text>
                        </div>
                        <div class="flex flex-wrap gap-2">
                            {move || {
                                radicals_stored
                                    .get_value()
                                    .unwrap_or_default()
                                    .into_iter()
                                    .map(|radical| {
                                        view! {
                                            <div class="flex items-center gap-1 px-2 py-1 bg-secondary/30 rounded">
                                                <Text size=TextSize::Large class="text-primary">
                                                    {radical.symbol}
                                                </Text>
                                                <div class="flex flex-col">
                                                    <Text size=TextSize::Small class="text-muted-foreground">
                                                        {radical.name}
                                                    </Text>
                                                    <Text size=TextSize::Small class="text-muted-foreground text-xs">
                                                        {radical.description}
                                                    </Text>
                                                </div>
                                            </div>
                                        }
                                    })
                                    .collect::<Vec<_>>()
                            }}
                        </div>
                    </div>
                </Show>

                <div class="flex gap-4 items-start text-left">
                    <div class="w-16 shrink-0">
                        <Text size=TextSize::Default variant=TypographyVariant::Muted>
                            "Написание"
                        </Text>
                    </div>
                    <KanjiWritingSection
                        kanji=kanji_stored.get_value()
                        mode=KanjiViewMode::Frames
                    />
                </div>
            </div>

            <Show when=move || examples_stored.get_value().is_some()>
                <div class="my-6">
                    <Text size=TextSize::Default variant=TypographyVariant::Muted class="mb-3 block text-left">
                        "Примеры слов:"
                    </Text>
                    <div class="grid grid-cols-2 sm:grid-cols-3 gap-3 text-left">
                        {move || {
                            examples_stored.get_value().map(|examples| {
                                examples
                                    .into_iter()
                                    .map(|(word, meaning)| {
                                        let meaning_stored = StoredValue::new(meaning);
                                        view! {
                                            <div class="p-2 bg-[var(--bg-aged)] rounded">
                                                <Text size=TextSize::Default class="font-bold">
                                                    <FuriganaText text=word known_kanji=known_kanji.get()/>
                                                </Text>
                                                <MarkdownText
                                                    content=Signal::derive(move || meaning_stored.get_value())
                                                    variant=MarkdownVariant::Compact
                                                    class="text-[var(--fg-muted)]"
                                                    known_kanji=known_kanji.get()
                                                />
                                            </div>
                                        }
                                    })
                                    .collect::<Vec<_>>()
                            })
                        }}
                    </div>
                </div>
            </Show>
        </Show>
    }
}
