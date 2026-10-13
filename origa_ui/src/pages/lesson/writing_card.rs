use super::card_type::CardType;
use super::kanji_card_data::extract_kanji_card_data;
use super::lesson_card_header::LessonCardTags;
use super::lesson_card_shell::LessonCardShell;
use crate::i18n::*;
use crate::pages::lesson::kanji_card_details::KanjiCardDetails;
use crate::pages::lesson::rating_buttons_view::RatingButtonsView;
use crate::pages::shared::format_answer_text;
use crate::ui_components::{DisplayText, Heading, HeadingLevel, KanjiDrawingPractice};
use leptos::prelude::*;
use origa::domain::{Card as DomainCard, NativeLanguage, Rating};
use std::collections::HashSet;
use tracing::warn;

#[component]
pub fn WritingCard(
    card: DomainCard,
    on_rate: Callback<Rating>,
    #[prop(optional)] on_show_answer: Option<Callback<()>>,
    #[prop(into)] disabled: Signal<bool>,
    native_language: NativeLanguage,
    #[prop(into)] known_kanji: Signal<HashSet<char>>,
) -> impl IntoView {
    let i18n = use_i18n();

    let (symbol_char, on_readings, kun_readings, radicals, example_words) =
        match extract_kanji_card_data(&card, native_language) {
            Some(data) => (
                data.symbol,
                data.on_readings,
                data.kun_readings,
                data.radicals,
                data.example_words,
            ),
            None => {
                return view! { <div>{t!(i18n, lesson.writing_card_kanji_only)}</div> }.into_any();
            },
        };

    let display_text = match &card {
        DomainCard::Kanji(kanji) => match kanji.description(&native_language) {
            Ok(_) => format_answer_text(&card, &native_language),
            Err(e) => {
                warn!(kanji = %symbol_char, error = %e, "Failed to get kanji description");
                String::new()
            },
        },
        _ => String::new(),
    };

    let show_details = RwSignal::new(false);

    let symbol_sv = StoredValue::new(symbol_char);
    let display_text_sv = StoredValue::new(display_text);
    let on_readings_sv = StoredValue::new(on_readings);
    let kun_readings_sv = StoredValue::new(kun_readings);
    let radicals_sv = StoredValue::new(radicals);
    let examples_sv = StoredValue::new(example_words);
    let on_rate_sv = StoredValue::new(on_rate);
    let disabled_sv = StoredValue::new(disabled);

    let card_for_tags = card.clone();
    let tags_row: ChildrenFn = std::sync::Arc::new(move || {
        view! {
            <LessonCardTags
                card_type=CardType::Kanji
                grammar_info=None
                show_answer=Signal::derive(|| false)
                card=card_for_tags.clone()
            />
        }
        .into_any()
    });

    view! {
        <LessonCardShell
            tags=tags_row
            test_id="lesson-card-root"
        >
        <div class="text-center mb-4">
            <DisplayText>{display_text_sv.get_value()}</DisplayText>
        </div>

        <Show when=move || !show_details.get()>
            <div class="my-4">
                <KanjiDrawingPractice
                    kanji=symbol_sv.get_value()
                    on_complete=Callback::new(move |_| {
                        show_details.set(true);
                        if let Some(cb) = on_show_answer {
                            cb.run(());
                        }
                    })
                />
            </div>
        </Show>

        <Show when=move || show_details.get()>
            <Heading level=HeadingLevel::H1 class="text-6xl mb-2 text-primary text-center">
                {symbol_sv.get_value()}
            </Heading>

            <KanjiCardDetails
                kanji=symbol_sv.get_value()
                name=display_text_sv.get_value()
                radicals=radicals_sv.get_value()
                example_words=examples_sv.get_value()
                on_readings=on_readings_sv.get_value()
                kun_readings=kun_readings_sv.get_value()
                known_kanji=known_kanji
                native_language=native_language
            />

            <div class="flex justify-center mt-4">
                <button
                    type="button"
                    class="btn btn-ghost text-xs"
                    on:click=move |_: leptos::ev::MouseEvent| {
                        show_details.set(false);
                    }
                >
                    {t!(i18n, lesson.writing_try_again)}
                </button>
            </div>

            <RatingButtonsView on_rate=on_rate_sv.get_value() disabled=disabled_sv.get_value() />
        </Show>
        </LessonCardShell>
    }
    .into_any()
}
