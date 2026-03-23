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
                            "Написание"
                        </Text>
                    </div>
                    <KanjiAnimation
                        kanji={radical_stored.get_value().symbol.to_string()}
                        mode=KanjiViewMode::Frames
                    />
                </div>

            </div>
        </Show>
    }
}
