use crate::pages::lesson::kanji_card_details::KanjiCardDetails;
use crate::pages::lesson::rating_buttons_view::RatingButtonsView;
use crate::pages::lesson::writing_card_details::RadicalDetailsView;
use crate::ui_components::{Card, DisplayText, KanjiDrawingPractice, Tag, TagVariant};
use gloo_timers::future::TimeoutFuture;
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::domain::{Card as DomainCard, NativeLanguage, Rating};
use std::collections::HashSet;
use tracing::warn;

#[derive(Clone)]
pub struct RadicalInfoForWriting {
    pub stroke_count: u32,
    pub kanji_examples: Option<Vec<char>>,
}

struct KanjiData {
    symbol: String,
    description: String,
    on_readings: Option<Vec<String>>,
    kun_readings: Option<Vec<String>>,
    radicals: Option<Vec<crate::pages::lesson::kanji_card_details::RadicalDisplay>>,
    examples: Option<Vec<(String, String)>>,
}

fn extract_kanji_data(kanji: &DomainCard, native_language: NativeLanguage) -> KanjiData {
    let DomainCard::Kanji(kanji) = kanji else {
        unreachable!()
    };

    let symbol = kanji.kanji().text().to_string();
    let description = kanji
        .description()
        .ok()
        .map(|d| d.text().to_string())
        .unwrap_or_default();

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

    let radicals: Option<Vec<crate::pages::lesson::kanji_card_details::RadicalDisplay>> =
        match kanji.radicals_info() {
            Ok(r) => Some(
                r.iter()
                    .map(
                        |info| crate::pages::lesson::kanji_card_details::RadicalDisplay {
                            symbol: info.radical(),
                            name: info.name().to_string(),
                            description: info.description().to_string(),
                        },
                    )
                    .collect(),
            ),
            Err(e) => {
                warn!("Failed to get radicals for kanji: {:?}", e);
                None
            },
        };

    let examples: Option<Vec<(String, String)>> = {
        let ex: Vec<_> = kanji
            .example_words(&native_language)
            .iter()
            .map(|e| (e.word().to_string(), e.meaning().to_string()))
            .collect();
        if ex.is_empty() { None } else { Some(ex) }
    };

    KanjiData {
        symbol,
        description,
        on_readings,
        kun_readings,
        radicals,
        examples,
    }
}

struct RadicalData {
    symbol: String,
    display_text: String,
    info: RadicalInfoForWriting,
}

fn extract_radical_data(radical: &DomainCard) -> RadicalData {
    let DomainCard::Radical(radical) = radical else {
        unreachable!()
    };

    let symbol = radical.radical_char().to_string();
    let radical_info_stored = radical.radical_info().ok();

    let display_text = radical_info_stored
        .as_ref()
        .map(|i| i.name().to_string())
        .unwrap_or_default();

    let stroke_count = radical_info_stored
        .as_ref()
        .map(|i| i.stroke_count())
        .unwrap_or(0);

    let kanji_examples: Option<Vec<char>> = {
        let examples = radical.kanji_examples();
        if examples.is_empty() {
            None
        } else {
            Some(examples)
        }
    };

    RadicalData {
        symbol,
        display_text,
        info: RadicalInfoForWriting {
            stroke_count,
            kanji_examples,
        },
    }
}

fn get_card_type(card: &DomainCard) -> (&'static str, TagVariant) {
    match card {
        DomainCard::Kanji(_) => ("Кандзи", TagVariant::Olive),
        DomainCard::Radical(_) => ("Радикал", TagVariant::Default),
        _ => ("Кандзи", TagVariant::Olive),
    }
}

#[component]
pub fn WritingCard(
    card: DomainCard,
    on_rate: Callback<Rating>,
    #[prop(into)] disabled: Signal<bool>,
    native_language: NativeLanguage,
    #[prop(into)] known_kanji: Signal<HashSet<String>>,
) -> impl IntoView {
    let (card_type_label, tag_variant) = get_card_type(&card);

    let (
        symbol_char,
        display_text,
        on_readings,
        kun_readings,
        radicals,
        example_words,
        radical_info,
    ) = match &card {
        DomainCard::Kanji(_) => {
            let data = extract_kanji_data(&card, native_language);
            (
                data.symbol,
                data.description,
                data.on_readings,
                data.kun_readings,
                data.radicals,
                data.examples,
                None,
            )
        },
        DomainCard::Radical(_) => {
            let data = extract_radical_data(&card);
            (
                data.symbol,
                data.display_text,
                None,
                None,
                None,
                None,
                Some(data.info),
            )
        },
        _ => {
            return view! { <div>"WritingCard поддерживает только кандзи и радикалы"</div> }
                .into_any();
        },
    };

    let show_details = RwSignal::new(false);
    let show_drawing = RwSignal::new(true);
    let is_expanded = RwSignal::new(true);

    Effect::new(move |_| {
        if show_details.get() {
            spawn_local(async move {
                TimeoutFuture::new(1500).await;
                show_drawing.set(false);
            });
        }
    });

    let symbol_sv = StoredValue::new(symbol_char);
    let display_text_sv = StoredValue::new(display_text);
    let on_readings_sv = StoredValue::new(on_readings);
    let kun_readings_sv = StoredValue::new(kun_readings);
    let radicals_sv = StoredValue::new(radicals);
    let examples_sv = StoredValue::new(example_words);
    let radical_info_sv = StoredValue::new(radical_info);
    let on_rate_sv = StoredValue::new(on_rate);
    let disabled_sv = StoredValue::new(disabled);

    view! {
        <Card class="p-4 sm:p-6 min-h-[250px] sm:min-h-[300px] flex flex-col" shadow=true>
            <div class="flex items-center justify-between mb-4">
                <Tag variant=tag_variant>
                    {card_type_label}
                </Tag>
            </div>

            <div class="flex-1 flex flex-col justify-center">
                <div class="text-center mb-4">
                    <DisplayText>{display_text_sv.get_value()}</DisplayText>
                </div>

                <Show when=move || show_drawing.get() && !show_details.get()>
                    <div class="my-4">
                        <KanjiDrawingPractice
                            kanji=symbol_sv.get_value()
                            on_complete=Callback::new(move |_| {
                                show_details.set(true);
                            })
                        />
                    </div>
                </Show>

                <Show when=move || show_details.get()>
                    <Show when=move || radicals_sv.get_value().is_some()>
                        <KanjiCardDetails
                            kanji=symbol_sv.get_value()
                            radicals=radicals_sv.get_value()
                            example_words=examples_sv.get_value()
                            show_details=is_expanded
                            on_readings=on_readings_sv.get_value()
                            kun_readings=kun_readings_sv.get_value()
                            known_kanji=known_kanji
                        />
                    </Show>

                    <Show when=move || radical_info_sv.get_value().is_some()>
                        <RadicalDetailsView
                            stroke_count=radical_info_sv
                                .get_value()
                                .as_ref()
                                .map(|i| i.stroke_count)
                                .unwrap_or(0)
                            kanji_examples=StoredValue::new(
                                radical_info_sv
                                    .get_value()
                                    .and_then(|i| i.kanji_examples.clone())
                            )
                        />
                    </Show>

                    <RatingButtonsView on_rate=on_rate_sv.get_value() disabled=disabled_sv.get_value() />
                </Show>
            </div>
        </Card>
    }
    .into_any()
}
