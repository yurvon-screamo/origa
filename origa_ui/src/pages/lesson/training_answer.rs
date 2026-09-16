use super::acquaintance_state::{AcquaintanceContext, AcquaintanceSlideData};
use super::grammar_example::first_example_markdown;
use crate::i18n::*;
use crate::ui_components::{FuriganaText, MarkdownText, MarkdownVariant, ReadingGroup};
use leptos::prelude::*;
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
    let i18n = use_i18n();
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
                        name,
                        on_readings,
                        kun_readings,
                        ..
                    } => {
                        let has_on = on_readings.is_some();
                        let has_kun = kun_readings.is_some();
                        let on = StoredValue::new(on_readings);
                        let kun = StoredValue::new(kun_readings);
                        // Знак уже смотрит на юзера сжатым вопросом над
                        // divider — ответ его не повторяет (баг-репорт о
                        // дубле): главным текстом идёт значение, под ним
                        // частотные чтения по центральной оси карточки.
                        view! {
                            <p class="font-serif text-3xl text-[var(--fg-black)]">{name}</p>
                            <div class="answer-readings pt-2 space-y-2">
                                {has_on.then(|| {
                                    view! {
                                        <ReadingGroup
                                            label=Signal::derive(move || {
                                                i18n.get_keys().lesson().on_yomi().inner().to_string()
                                            })
                                            readings=on
                                        />
                                    }
                                })}
                                {has_kun.then(|| {
                                    view! {
                                        <ReadingGroup
                                            label=Signal::derive(move || {
                                                i18n.get_keys().lesson().kun_yomi().inner().to_string()
                                            })
                                            readings=kun
                                        />
                                    }
                                })}
                            </div>
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
                                    content=Signal::derive(move || examples_stored.get_value())
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
