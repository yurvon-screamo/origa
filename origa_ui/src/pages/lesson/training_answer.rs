use super::acquaintance_state::{AcquaintanceContext, AcquaintanceSlideData};
use super::grammar_details_expand::GrammarDetailsExpand;
use super::grammar_example::first_example_markdown;
use super::kanji_card_details::KanjiCardDetails;
use crate::ui_components::{
    FuriganaText, MarkdownText, MarkdownVariant, example_fences_to_paragraphs,
};
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
                        rule_id,
                        pattern,
                        short_description,
                        examples,
                        ..
                    } => {
                        // Фронт — JP-пример; при пустых examples фронт —
                        // паттерн («знак правила», training_front), и ответ
                        // не дублирует его. Заголовок ответа — локализованное
                        // описание (sd), паттерн — вторичная mono-строка
                        // (#503 UX иерархия).
                        let example = first_example_markdown(&examples);
                        let front_was_pattern = example.is_none();
                        let examples_stored = StoredValue::new(example.unwrap_or_default());
                        let pattern_stored = StoredValue::new(pattern);
                        // «Подробнее» — тот же компонент, что в обычном уроке
                        // (lesson_card_answer): полный разбор правила по
                        // rule_id. Кнопка живёт независимо от loaded-стиля
                        // словаря; тело раскрывается по клику (K-итерация:
                        // раньше деталь правила была недоступна вовсе).
                        let is_grammar_expanded = RwSignal::new(false);
                        let kk_for_details = known_kanji.get_untracked();
                        let native_for_details = ctx.native_language.get_untracked();
                        view! {
                            <h2 class="font-serif text-2xl text-[var(--fg-black)]">
                                {short_description.clone()}
                            </h2>
                            <Show when=move || !front_was_pattern>
                                <p class="font-mono text-sm text-[var(--fg-muted)]">
                                    {pattern_stored.get_value()}
                                </p>
                            </Show>
                            <Show when=move || !front_was_pattern>
                                <MarkdownText
                                    content=Signal::derive(move || {
                                        example_fences_to_paragraphs(
                                            &examples_stored.get_value(),
                                        )
                                    })
                                    known_kanji=known_kanji.get_untracked()
                                    variant=Signal::derive(|| MarkdownVariant::Compact)
                                />
                            </Show>
                            <div class="text-left">
                                <GrammarDetailsExpand
                                    rule_id
                                    is_expanded=is_grammar_expanded
                                    known_kanji=kk_for_details
                                    native_language=native_for_details
                                    test_id=Signal::derive(|| {
                                        "acquaintance-grammar-details".to_string()
                                    })
                                />
                            </div>
                        }
                            .into_any()
                    },
                    // Счётный суффикс: ответ = значение + таблица чтений
                    // (issue #415). Таблица — общий компонент
                    // CounterReadingsTable (как в показе и композитном слоте).
                    AcquaintanceSlideData::Counter {
                        suffix,
                        meaning,
                        table,
                        ..
                    } => {
                        use super::counter_readings_table::{
                            CounterReadingRow, CounterReadingsTable,
                        };
                        let rows: Vec<CounterReadingRow> = table
                            .into_iter()
                            .map(|(number_label, reading, irregular)| CounterReadingRow {
                                number: 0,
                                number_label,
                                reading,
                                irregular,
                            })
                            .collect();
                        view! {
                            <div class="flex flex-col gap-4" data-testid="acquaintance-counter-answer">
                                <p class="font-mono text-lg text-[var(--fg-muted)] text-center">
                                    {meaning}
                                </p>
                                <CounterReadingsTable
                                    rows=rows
                                    suffix=suffix
                                    test_id=Signal::derive(|| "acquaintance-counter-table".to_string())
                                />
                            </div>
                        }
                            .into_any()
                    },
                }
            }}
        </div>
    }
}
