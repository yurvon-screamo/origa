use crate::ui_components::{
    Button, ButtonVariant, KanjiAnimation, KanjiDrawingPractice, KanjiViewMode, Text, TextSize,
    TypographyVariant,
};
use leptos::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct RadicalCardDisplay {
    pub symbol: char,
    pub name: String,
    pub description: String,
    pub stroke_count: u32,
    pub kanji_examples: Vec<char>,
}

#[component]
pub fn RadicalCardDetails(
    radical: RadicalCardDisplay,
    #[prop(into)] show_details: Signal<bool>,
) -> impl IntoView {
    let radical_stored = StoredValue::new(radical);
    let kanji_examples_stored = StoredValue::new(radical_stored.get_value().kanji_examples.clone());

    view! {
        <Show when=move || show_details.get()>
            <div class="my-6 space-y-4 max-w-max mx-auto">
                <div class="flex gap-4 items-start text-left">
                    <div class="w-16 shrink-0">
                        <Text size=TextSize::Default variant=TypographyVariant::Muted>
                            "Штрихи"
                        </Text>
                    </div>
                    <Text size=TextSize::Large>
                        {move || radical_stored.get_value().stroke_count.to_string()}
                    </Text>
                </div>

                <div class="flex gap-4 items-start text-left">
                    <div class="w-16 shrink-0">
                        <Text size=TextSize::Default variant=TypographyVariant::Muted>
                            "Название"
                        </Text>
                    </div>
                    <Text size=TextSize::Large>
                        {move || radical_stored.get_value().name.clone()}
                    </Text>
                </div>

                <div class="flex gap-4 items-start text-left">
                    <div class="w-16 shrink-0">
                        <Text size=TextSize::Default variant=TypographyVariant::Muted>
                            "Описание"
                        </Text>
                    </div>
                    <Text size=TextSize::Default>
                        {move || radical_stored.get_value().description.clone()}
                    </Text>
                </div>

                <div class="flex gap-4 items-start text-left">
                    <div class="w-16 shrink-0">
                        <Text size=TextSize::Default variant=TypographyVariant::Muted>
                            "Анимация"
                        </Text>
                    </div>
                    <KanjiAnimation
                        kanji={radical_stored.get_value().symbol.to_string()}
                        mode=KanjiViewMode::Animation
                    />
                </div>

                <div class="flex gap-4 items-start text-left">
                    <div class="w-16 shrink-0">
                        <Text size=TextSize::Default variant=TypographyVariant::Muted>
                            "Покадрово"
                        </Text>
                    </div>
                    <KanjiAnimation
                        kanji={radical_stored.get_value().symbol.to_string()}
                        mode=KanjiViewMode::Frames
                    />
                </div>

                <div class="flex gap-4 items-start text-left">
                    <div class="w-16 shrink-0">
                        <Text size=TextSize::Default variant=TypographyVariant::Muted>
                            "Прописи"
                        </Text>
                    </div>
                    <KanjiDrawingPractice
                        kanji={radical_stored.get_value().symbol.to_string()}
                    />
                </div>
            </div>

            <Show when=move || !kanji_examples_stored.get_value().is_empty()>
                <div class="my-6">
                    <Text size=TextSize::Default variant=TypographyVariant::Muted class="mb-3 block text-left">
                        "Примеры кандзи:"
                    </Text>
                    <div class="flex flex-wrap gap-2">
                        {move || {
                            kanji_examples_stored
                                .get_value()
                                .iter()
                                .map(|&kanji| {
                                    view! {
                                        <Button
                                            variant=ButtonVariant::Ghost
                                            class="text-xl px-3 py-2 min-w-[48px]"
                                        >
                                            {kanji.to_string()}
                                        </Button>
                                    }
                                })
                                .collect::<Vec<_>>()
                        }}
                    </div>
                </div>
            </Show>
        </Show>
    }
}
