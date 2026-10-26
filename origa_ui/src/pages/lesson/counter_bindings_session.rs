//! Пачка связок счётного суффикса (issue #415): каждая цифра × суффикс —
//! отдельный показ классическим «знаю / не знаю». Фронт — «8×本»,
//! ответ — чтение из реестра + глосса + таблица всех чтений с акцентом
//! отвеченной строки. Рейтинг уходит ТОЛЬКО в память этой связки
//! (`RateCounterBindingUseCase`, CounterReview); после последней связки
//! слот завершается штатным advance.
//!
//! Прогресс пачки живёт в ОБЩЕМ счётчике урока (юзер-репорт: вложенный
//! «k / N» неудобен) — сессия публикует свой шаг в
//! `LessonContext::counter_binding_step`, хедер сворачивает его в
//! `lesson-progress-text`; собственной шкалы у карточки нет.
//!
//! Компоновка — эталон прочих карточек урока: тег типа + кнопка
//! озвучки чтения в строке тегов НАД карточкой (кнопка — справа,
//! ответная сторона: фронт молчит, звук выдал бы ответ); тело карточки
//! центрировано (`flex-1 justify-center`); пара «Не знаю / Знаю» — ПОД
//! карточкой, как у RatingButtons остальных карт.
//!
//! testid-контракт: `counter-binding-answer` — контейнер ответа, его
//! текст чтения — `counter-binding-answer-reading` (чтение — ГЛАВНЫЙ
//! текст ответа вместо «число×суффикс», фуриганы/ruby нет — юзер-репорт);
//! кнопка повтора озвучки — `counter-binding-audio-btn` (в строке тегов).

use super::binary_rating_buttons::RatingButtons;
use super::card_type::CardType;
use super::counter_readings_table::{CounterReadingRow, CounterReadingsTable};
use super::lesson_card_header::LessonTagsRow;
use super::reveal_answer_button::RevealAnswerButton;
use crate::i18n::*;
use crate::ui_components::{AudioButtons, Button, ButtonVariant, Card, Tag, speak_word};
use leptos::prelude::*;
use leptos_use::use_event_listener;
use origa::domain::{Card, CounterBindingItem, Rating};

/// Гейт озвучки чтения связки при раскрытии (#808): мьют урока главнее,
/// пустое чтение (число вне реестра — защитный случай) молчит. Гейта
/// TTS-доступности нет сознательно: `speak_word` деградирует безопасно
/// (pitch-индекс → TTS-каны → подавленный `Err`), pitch-only среды
/// сохраняют CDN-звук.
fn should_speak_binding_reading(is_muted: bool, reading: &str) -> bool {
    !is_muted && !reading.is_empty()
}

/// Лейбл числа: 0 — вопросительное 何, прочие — само число
/// (лексикализованные исключения реестра несут свой порядок в таблице).
fn number_label(number: u8) -> String {
    match number {
        0 => "何".to_string(),
        n => n.to_string(),
    }
}

#[component]
pub fn CounterBindingsSession(
    card: Card,
    items: Vec<CounterBindingItem>,
    /// Оценка связки: (number, rating, is_last) — вызывающий подключает
    /// `RateCounterBindingUseCase` со своим репозиторием. `is_last`
    /// сериализует advance урока ПОСЛЕ сохранения рейтинга (юзер-репорт
    /// 0.8.0-rc: параллельный сейв семантики затирал рейтинг последней
    /// связки — 何 вечно возвращалась новичком, пачка мелькала каждый
    /// урок). Сама сессия урок не продвигает.
    on_rate_binding: Callback<(u8, Rating, bool)>,
    /// Мьют урока (тот же сигнал, что глушит автозвук остальных карт):
    /// озвучка чтения при раскрытии ему подчиняется.
    #[prop(into)]
    is_muted: Signal<bool>,
    #[prop(into)] test_id: Signal<String>,
) -> impl IntoView {
    let i18n = use_i18n();
    let card_type = CardType::Counter;
    let current = RwSignal::new(0usize);
    let revealed = RwSignal::new(false);
    // Завершение пачки: повторные 1/2/клики на замороженном ответе
    // игнорируются (двойной advance урока — перескок карточек).
    let finished = RwSignal::new(false);
    // Полная таблица чтений на ответе свёрнута по умолчанию (юзер-репорт:
    // «вижу лишь ответ; фулл таблицу — под развернуть»); кнопка — тот же
    // expand/collapse-паттерн, что у объёмных карточек ответа. Разворот
    // НЕ сбрасывается между связками пачки: раскрывший таблицу юзер
    // сохраняет контекст, акцент строки обновляется на каждом ответе.
    let table_expanded = RwSignal::new(false);
    // Акцент строки текущей связки (u8::MAX — вне диапазона чисел).
    let highlighted_number = RwSignal::new(u8::MAX);
    let total = items.len();
    // StoredValue (Copy): Fn-замыкания view и клавиатуры получают копию
    // хендла, не владея Vec (паттерн translator.rs).
    let items = StoredValue::new(items);

    // Публикация шага пачки в общий счётчик хедера (юзер-репорт: без
    // вложенного «k / N»); headless-монтирования без LessonContext
    // (wasm-тесты) работают как раньше — паттерн example_recall_card.
    let lesson_ctx = use_context::<super::LessonContext>();
    let step_ctx = lesson_ctx.clone();
    if let Some(ctx) = step_ctx.as_ref() {
        ctx.counter_binding_step.set(Some(0));
    }
    on_cleanup(move || {
        if let Some(ctx) = lesson_ctx.as_ref() {
            ctx.counter_binding_step.set(None);
        }
    });

    // Чтения и глосса — реестр (истина датасета).
    let entry = match &card {
        Card::Counter(counter) => origa::dictionary::counters::get_counter(counter.suffix()),
        _ => None,
    };
    let suffix = StoredValue::new(match &card {
        Card::Counter(counter) => counter.suffix().to_string(),
        _ => String::new(),
    });
    let rows: Vec<CounterReadingRow> = entry
        .map(|e| {
            e.readings()
                .iter()
                .map(|r| CounterReadingRow {
                    number: r.number(),
                    number_label: number_label(r.number()),
                    reading: r.reading().to_string(),
                    irregular: r.irregular(),
                })
                .collect()
        })
        .unwrap_or_default();
    let rows = StoredValue::new(rows);
    let gloss = StoredValue::new(
        entry
            .map(|e| {
                let lang = crate::i18n::locale_to_native_language(&i18n.get_locale_untracked());
                origa::dictionary::counters::gloss_for(e, lang).to_string()
            })
            .unwrap_or_default(),
    );
    let readings: StoredValue<Vec<(u8, String)>> =
        StoredValue::new(items.with_value(|list: &Vec<CounterBindingItem>| {
            list.iter()
                .map(|item| {
                    (
                        item.number(),
                        entry
                            .and_then(|e| e.reading_for(item.number()))
                            .unwrap_or("")
                            .to_string(),
                    )
                })
                .collect::<Vec<_>>()
        }));

    // Единственный резолв «текущая связка → чтение реестра»: ответ,
    // кнопка озвучки в строке тегов и озвучка reveal читают один сигнал
    // (рассинхрон трёх копий озвучил бы не то чтение, что на экране).
    let current_reading = Signal::derive(move || {
        let number = items
            .with_value(|i| i.get(current.get()).map(|it| it.number()))
            .unwrap_or(u8::MAX);
        readings
            .with_value(|rs| {
                rs.iter()
                    .find(|(n, _)| *n == number)
                    .map(|(_, r)| r.clone())
            })
            .unwrap_or_default()
    });

    let rate = move |rating: Rating| {
        if !revealed.get_untracked() || finished.get_untracked() {
            return;
        }
        let index = current.get_untracked();
        let Some(item) = items.with_value(|i| i.get(index).cloned()) else {
            return;
        };
        let is_last = index + 1 >= total;
        on_rate_binding.run((item.number(), rating, is_last));
        if is_last {
            // Последняя связка: отвеченная карта ОСТАЁТСЯ на экране
            // (замороженный ответ — сброс revealed мелькал фронтом перед
            // следующей карточкой урока). Advance выполняет контейнер
            // ПОСЛЕ сохранения рейтинга (is_last) — параллельные
            // get+save больше не конкурируют за юзера. Повторные
            // 1/2/клики гасятся finished-гейтом выше.
            finished.set(true);
        } else {
            revealed.set(false);
            highlighted_number.set(u8::MAX);
            current.set(index + 1);
            // Общий счётчик шагает вместе с пачкой (вложенной шкалы
            // больше нет — шаг уходит в хедер).
            if let Some(ctx) = step_ctx.as_ref() {
                ctx.counter_binding_step.set(Some(index + 1));
            }
        }
    };
    let on_rate = Callback::new(move |rating: Rating| rate(rating));

    let reveal = move || {
        if revealed.get_untracked() || finished.get_untracked() {
            return;
        }
        let number = items
            .with_value(|i| i.get(current.get_untracked()).map(|item| item.number()))
            .unwrap_or(u8::MAX);
        revealed.set(true);
        highlighted_number.set(number);
        // Озвучка канонического чтения связки (#808): чтение из реестра —
        // чистая кана, поэтому speak_word резолвит его корректно (pitch-
        // индекс по возможности, иначе TTS читает кану как есть). Кнопка и
        // Space/Enter ведут в этот же reveal — один канал звука. Фронт
        // связки НЕ озвучивается: звучащее чтение выдало бы ответ.
        let reading = current_reading.get_untracked();
        if should_speak_binding_reading(is_muted.get_untracked(), &reading) {
            speak_word(&reading, 1.0);
        }
    };

    // Клавиатура слота: Space — показ ответа, 1/2 — «Не знаю»/«Знаю»
    // (тот же контракт, что у остальных карт).
    let _ = use_event_listener(document(), leptos::ev::keydown, move |ev| {
        if ev.repeat() {
            return;
        }
        if super::keyboard_handler::is_typing_target(ev.target().as_ref()) {
            return;
        }
        match ev.key().as_str() {
            " " | "Enter" if !revealed.get_untracked() => {
                ev.prevent_default();
                reveal();
            },
            "1" if revealed.get_untracked() => {
                ev.prevent_default();
                on_rate.run(Rating::Again);
            },
            "2" if revealed.get_untracked() => {
                ev.prevent_default();
                on_rate.run(Rating::Good);
            },
            _ => {},
        }
    });

    view! {
        // Строка тегов над карточкой — общий паттерн урока: тег типа
        // слева; кнопка повтора озвучки чтения — справа (top-right, как
        // у остальных карт; юзер-репорт про «плеер по центру»). Кнопка
        // только на ответе и только вне мьюта: фронт молчит (звук раскрыл
        // бы ответ), мьют главнее (тот же контракт, что до реструктуризации).
        <LessonTagsRow>
            <Tag variant=Signal::derive(move || card_type.tag_variant())>
                {card_type.label(&i18n)}
            </Tag>
            <Show when=move || revealed.get() && !is_muted.get()>
                {move || {
                    // AudioButtons не берёт реактивный text — компонент
                    // пересоздаётся замыканием на каждой смене связки.
                    let reading = current_reading.get();
                    view! {
                        <div class="ml-auto">
                            <AudioButtons
                                text=reading
                                audio_path=None
                                test_id=Signal::derive(|| "counter-binding-audio-btn".to_string())
                            />
                        </div>
                    }
                }}
            </Show>
        </LessonTagsRow>

        <Card class=Signal::derive(|| super::LESSON_CARD_CLASS.to_string()) shadow=true test_id=test_id>
            <div class="flex-1 flex flex-col justify-center text-center gap-4">
                <Show when=move || !revealed.get()>
                    <div class="flex items-center justify-center gap-2" data-testid="counter-binding-front">
                        <span class="font-mono text-5xl text-[var(--fg-black)]">
                            {move || number_label(items.with_value(|i| i.get(current.get()).map(|it| it.number())).unwrap_or(u8::MAX))}
                        </span>
                        <span class="text-3xl text-[var(--fg-muted)]">{"×"}</span>
                        <span class="font-serif text-7xl text-[var(--fg-black)]">
                            {suffix.with_value(String::clone)}
                        </span>
                    </div>
                    <p class="font-mono text-[var(--fg-muted)]">{gloss.get_value()}</p>
                    <div class="flex justify-center">
                        <RevealAnswerButton
                            test_id="counter-binding-show-answer-btn"
                            on_click=Callback::new(move |_: ()| reveal())
                        />
                    </div>
                </Show>

                {move || {
                    if !revealed.get() {
                        return ().into_any();
                    }
                    let reading = current_reading.get();
                    view! {
                        // Ответ: ГЛАВНЫЙ текст — чтение из реестра (чистая
                        // кана) вместо «число×суффикс» с ruby-фуриганой
                        // (юзер-репорт: фуригану не хочу, хочу только
                        // чтение). Число×суффикс уже был фронтом; чтение
                        // конкатенацией TTS дала бы неверное произношение
                        // (10本 → «じゅうほん» вместо «じゅっぽん») — только
                        // реестр.
                        <p
                            class="font-serif text-6xl text-[var(--fg-black)] break-words"
                            data-testid="counter-binding-answer"
                        >
                            <span data-testid="counter-binding-answer-reading">
                                {reading}
                            </span>
                        </p>
                        <p class="font-mono text-[var(--fg-muted)]">{gloss.get_value()}</p>

                        // Полная таблица чтений — под «Развернуть»
                        // (expand/collapse как у объёмных ответов урока);
                        // по умолчанию юзер видит только ответ.
                        <Show when=move || table_expanded.get()>
                            <div data-testid="counter-binding-mutations">
                                <CounterReadingsTable
                                    rows=rows.get_value()
                                    suffix=suffix.with_value(String::clone)
                                    highlight=highlighted_number
                                    test_id=Signal::derive(|| "counter-binding-mutations-table".to_string())
                                />
                            </div>
                        </Show>
                        <div class="flex justify-center">
                            <Button
                                variant=ButtonVariant::Ghost
                                on_click=Callback::new(move |_| table_expanded.update(|v| *v = !*v))
                                test_id=Signal::derive(|| "counter-binding-mutations-toggle".to_string())
                            >
                                {move || if table_expanded.get() { t!(i18n, common.collapse).into_any() } else { t!(i18n, common.expand).into_any() }}
                            </Button>
                        </div>
                    }
                    .into_any()
                }}
            </div>
        </Card>

        // Пара «Не знаю / Знаю» ПОД карточкой — эталон RatingButtons
        // остальных карт (юзер-репорт: кнопки почти по центру карточки).
        // На замороженном последнем ответе остаются видимыми: клики гасит
        // finished-гейт rate().
        <Show when=move || revealed.get()>
            <RatingButtons on_rate=on_rate />
        </Show>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    /// #808: мьют глушит озвучку раскрытой связки; пустое чтение (число
    /// вне реестра) молчит; каноническое чтение при немьюте звучит.
    #[rstest]
    #[case(false, "いっぽん", true)]
    #[case(true, "いっぽん", false)]
    #[case(false, "", false)]
    #[case(true, "", false)]
    fn speak_gate_by_mute_and_reading(
        #[case] is_muted: bool,
        #[case] reading: &str,
        #[case] expected: bool,
    ) {
        assert_eq!(should_speak_binding_reading(is_muted, reading), expected);
    }
}
