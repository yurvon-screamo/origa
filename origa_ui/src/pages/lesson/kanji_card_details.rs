use crate::ui_components::{
    FuriganaText, KanjiViewMode, KanjiWritingSection, MarkdownText, MarkdownVariant, Text,
    TextSize, TypographyVariant,
};
use leptos::prelude::*;
use origa::domain::User;

#[component]
pub fn KanjiCardDetails(
    kanji: String,
    radicals: Option<String>,
    example_words: Option<Vec<(String, String)>>,
    show_details: bool,
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

    view! {
        <Show when=move || show_details>
            <KanjiWritingSection kanji=kanji_stored.get_value() mode=KanjiViewMode::Frames />

            <Show when=move || radicals_stored.get_value().is_some()>
                <div class="my-6">
                    <Text size=TextSize::Default variant=TypographyVariant::Muted>
                        {format!("Радикалы: {}", radicals_stored.get_value().unwrap_or_default())}
                    </Text>
                </div>
            </Show>

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
