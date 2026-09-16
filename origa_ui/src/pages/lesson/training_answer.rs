use super::acquaintance_state::{AcquaintanceContext, AcquaintanceSlideData};
use super::grammar_example::first_example_markdown;
use super::kanji_card_details::KanjiCardDetails;
use crate::ui_components::{FuriganaText, MarkdownText, MarkdownVariant};
use leptos::prelude::*;
use std::collections::HashSet;
use ulid::Ulid;

/// Ответ тренировки: противоположная фронту сторона. Для слов Reverse —
/// слово с фуриганой и повтор аудио (спека §8.2); кандзи раскрывают
/// значения и частотные чтения; грамматика — смысл с полным примером.
#[component]
pub(super) fn TrainingAnswerSlide(
    ctx: AcquaintanceContext,
    card_id: Ulid,
    reverse: bool,
) -> impl IntoView {
    let known_kanji = ctx.known_kanji;
    view! {
        <div class="text-center space-y-3" data-testid="acquaintance-training-answer">
            {move || {
                let Some(slide) = ctx
                    .slides
                    .get()
                    .iter()
                    .find(|s| s.card_id() == card_id)
                    .cloned()
                else {
                    return ().into_any();
                };
                match slide {
                    AcquaintanceSlideData::Vocabulary { word, translations, .. } => {
                        if reverse {
                            view! {
                                <p class="font-serif text-5xl text-[var(--fg-black)] break-words">
                                    <FuriganaText
                                        text=word
                                        known_kanji=known_kanji.get_untracked()
                                        native_language=ctx.native_language.get_untracked()
                                        with_kanji_tooltip=true
                                    />
                                </p>
                            }
                                .into_any()
                        } else if ctx.audio_front.get_untracked() {
                            // Аудио-фронт: слова на вопросе не было (только
                            // звук) — ответ показывает слово и перевод, как
                            // текстовый фронт после раскрытия. Повтор
                            // доступен кнопкой в шапке (владелец, 2026-09-16).
                            view! {
                                <p class="font-serif text-5xl text-[var(--fg-black)] break-words">
                                    <FuriganaText
                                        text=word
                                        known_kanji=known_kanji.get_untracked()
                                        native_language=ctx.native_language.get_untracked()
                                        with_kanji_tooltip=true
                                    />
                                </p>
                                <p class="font-mono text-2xl text-[var(--fg-black)] pt-1">
                                    {translations.join(", ")}
                                </p>
                            }
                                .into_any()
                        } else {
                            view! {
                                <p class="font-mono text-2xl text-[var(--fg-black)]">
                                    {translations.join(", ")}
                                </p>
                            }
                                .into_any()
                        }
                    },
                    AcquaintanceSlideData::Kanji {
                        kanji,
                        name,
                        radicals,
                        example_words,
                        on_readings,
                        kun_readings,
                        ..
                    } => {
                        // Знак уже смотрит на юзера сжатым вопросом над
                        // divider — ответ его не повторяет: тот же блок
                        // деталей, что в обычном уроке и показе руки
                        // (чтения, значение, свёрнутые «Подробнее»).
                        let known_kanji_signal: Signal<HashSet<char>> = known_kanji.into();
                        view! {
                            <KanjiCardDetails
                                kanji=kanji
                                name
                                radicals
                                example_words
                                on_readings
                                kun_readings
                                known_kanji=known_kanji_signal
                                native_language=ctx.native_language.get_untracked()
                            />
                        }
                            .into_any()
                    },
                    AcquaintanceSlideData::Grammar {
                        title,
                        short_description,
                        examples,
                        ..
                    } => {
                        // Фронт — JP-пример; при пустых examples фронт был
                        // заголовком конструкции, и ответ не дублирует его.
                        let example = first_example_markdown(&examples);
                        let front_was_title = example.is_none();
                        let examples_stored = StoredValue::new(example.unwrap_or_default());
                        let title_stored = StoredValue::new(title);
                        view! {
                            <Show when=move || !front_was_title>
                                <h2 class="font-serif text-2xl text-[var(--fg-black)]">
                                    {title_stored.get_value()}
                                </h2>
                            </Show>
                            <p class="font-mono text-sm">{short_description}</p>
                            <Show when=move || !front_was_title>
                                <MarkdownText
                                    content=Signal::derive(move || {
                                        crate::ui_components::example_fences_to_paragraphs(
                                            &examples_stored.get_value(),
                                        )
                                    })
                                    known_kanji=known_kanji.get_untracked()
                                    variant=Signal::derive(|| MarkdownVariant::Compact)
                                />
                            </Show>
                        }
                            .into_any()
                    },
                }
            }}
        </div>
    }
}
