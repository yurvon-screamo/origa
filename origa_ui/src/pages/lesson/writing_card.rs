use crate::pages::lesson::kanji_card_details::{KanjiCardDetails, RadicalDisplay};
use crate::pages::lesson::rating_buttons_view::RatingButtonsView;
use crate::ui_components::{
    Card, DisplayText, KanjiDrawingPractice, ReadingGroup, Tag, TagVariant, Text, TextSize,
    TypographyVariant,
};
use leptos::prelude::*;
use origa::domain::{Card as DomainCard, NativeLanguage, Rating};
use std::collections::HashSet;
use tracing::warn;

#[component]
pub fn WritingCard(
    card: DomainCard,
    on_rate: Callback<Rating>,
    #[prop(into)] disabled: Signal<bool>,
    native_language: NativeLanguage,
    #[prop(into)] known_kanji: Signal<HashSet<String>>,
) -> impl IntoView {
    let DomainCard::Kanji(kanji) = &card else {
        return view! { <div>"WritingCard поддерживает только кандзи"</div> }.into_any();
    };

    let kanji_char = kanji.kanji().text().to_string();
    let on_readings: Option<Vec<String>> = {
        let readings = kanji.on_readings();
        if readings.is_empty() {
            None
        } else {
            Some(readings)
        }
    };
    let kun_readings: Option<Vec<String>> = {
        let readings = kanji.kun_readings();
        if readings.is_empty() {
            None
        } else {
            Some(readings)
        }
    };

    let radicals: Option<Vec<RadicalDisplay>> = match kanji.radicals_info() {
        Ok(r) => Some(
            r.iter()
                .map(|info| RadicalDisplay {
                    symbol: info.radical(),
                    name: info.name().to_string(),
                    description: info.description().to_string(),
                })
                .collect(),
        ),
        Err(e) => {
            warn!("Failed to get radicals for kanji: {:?}", e);
            None
        }
    };

    let example_words: Option<Vec<(String, String)>> = {
        let examples: Vec<_> = kanji
            .example_words(&native_language)
            .iter()
            .map(|e| (e.word().to_string(), e.meaning().to_string()))
            .collect();
        if examples.is_empty() {
            None
        } else {
            Some(examples)
        }
    };

    let is_completed = RwSignal::new(false);
    let is_expanded = RwSignal::new(true);

    let kanji_stored = StoredValue::new(kanji_char.clone());
    let on_readings_stored = StoredValue::new(on_readings);
    let kun_readings_stored = StoredValue::new(kun_readings);
    let radicals_stored = StoredValue::new(radicals);
    let examples_stored = StoredValue::new(example_words);

    let on_complete = Callback::new(move |_| {
        is_completed.set(true);
    });

    let kanji_for_drawing = kanji_char.clone();

    view! {
        <Card class=Signal::derive(|| "p-4 sm:p-6 min-h-[250px] sm:min-h-[300px] flex flex-col".to_string()) shadow=Signal::derive(|| true)>
            <div class="flex items-center justify-between mb-4">
                <Tag variant=Signal::derive(|| TagVariant::Olive)>"Кандзи"</Tag>
            </div>

            <div class="flex-1 flex flex-col justify-center">
                <div class="text-center mb-4">
                    <DisplayText>{kanji_char.clone()}</DisplayText>
                </div>

                <div class="mb-4 space-y-2 max-w-max mx-auto">
                    <ReadingGroup label="音読み[онъёми]" readings=on_readings_stored />
                    <ReadingGroup label="訓読み[кунъёми]" readings=kun_readings_stored />
                </div>

                <div class="mb-4">
                    <Text size=TextSize::Default variant=TypographyVariant::Muted class="text-center block mb-2">
                        "Попробуйте написать кандзи:"
                    </Text>
                    <KanjiDrawingPractice kanji=kanji_for_drawing on_complete=on_complete />
                </div>

                <Show when=move || is_completed.get()>
                    <KanjiCardDetails
                        kanji=kanji_stored.get_value()
                        radicals=radicals_stored.get_value()
                        example_words=examples_stored.get_value()
                        show_details=is_expanded.get()
                        on_readings=on_readings_stored.get_value()
                        kun_readings=kun_readings_stored.get_value()
                        known_kanji=known_kanji
                    />

                    <RatingButtonsView on_rate disabled />
                </Show>
            </div>
        </Card>
    }
    .into_any()
}
