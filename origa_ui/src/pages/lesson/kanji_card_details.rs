use crate::ui_components::{
    FuriganaText, KanjiViewMode, KanjiWritingSection, MarkdownText, MarkdownVariant, ReadingGroup,
    Text, TextSize, TypographyVariant,
};
use leptos::prelude::*;
use origa::domain::User;

#[component]
pub fn KanjiCardDetails(
    kanji: String,
    radicals: Option<String>,
    example_words: Option<Vec<(String, String)>>,
    show_details: bool,
    on_readings: Option<Vec<String>>,
    kun_readings: Option<Vec<String>>,
) -> impl IntoView {
    let current_user = use_context::<RwSignal<Option<User>>>().expect("current_user context");

    let known_kanji = Memo::new(move |_| {
        current_user
            .get()
            .map(|u| u.knowledge_set().get_known_kanji())
            .unwrap_or_default()
    });

    let kanji_stored = StoredValue::new(kanji);
    let radicals_stored = StoredValue::new(radicals);
    let examples_stored = StoredValue::new(example_words);
    let on_readings_stored = StoredValue::new(on_readings);
    let kun_readings_stored = StoredValue::new(kun_readings);

    view! {
        <Show when=move || show_details>
            <KanjiWritingSection kanji=kanji_stored.get_value() mode=KanjiViewMode::Frames />

            <div class="my-6 space-y-4 max-w-max mx-auto">
                <ReadingGroup label="音読み" readings=on_readings_stored />
                <ReadingGroup label="訓読み" readings=kun_readings_stored />

                <Show when=move || radicals_stored.get_value().is_some()>
                    <div class="flex gap-4 items-start text-left">
                        <div class="w-16 shrink-0">
                            <Text size=TextSize::Default variant=TypographyVariant::Muted>
                                "Радикалы"
                            </Text>
                        </div>
                        <Text size=TextSize::Default>
                            {radicals_stored.get_value().unwrap_or_default()}
                        </Text>
                    </div>
                </Show>
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
                                            <div class="p-2 bg-[var(--bg-secondary)] rounded">
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
