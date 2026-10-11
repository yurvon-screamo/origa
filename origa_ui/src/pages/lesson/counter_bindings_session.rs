//! Пачка связок счётного суффикса (issue #415): каждая цифра × суффикс —
//! отдельный показ классическим «знаю / не знаю». Фронт — «8×本»,
//! ответ — чтение + глосса + таблица всех чтений с акцентом отвеченной
//! строки. Рейтинг уходит ТОЛЬКО в память этой связки
//! (`RateCounterBindingUseCase`, CounterReview); после последней связки
//! слот завершается штатным advance.
//!
//! testid-контракт: `counter-binding-answer` — заголовок-контейнер
//! ответа («число×суффикс»); текст чтения —
//! `counter-binding-answer-reading` (не читать чтение с заголовка).

use super::counter_readings_table::{CounterReadingRow, CounterReadingsTable};
use super::rating_buttons_view::RatingButtonsView;
use crate::i18n::*;
use crate::ui_components::{Button, ButtonVariant, Card};
use leptos::prelude::*;
use leptos_use::use_event_listener;
use origa::domain::{Card, CounterBindingItem, Rating};

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
    /// Оценка связки: (number, rating) — вызывающий подключает
    /// `RateCounterBindingUseCase` со своим репозиторием.
    on_rate_binding: Callback<(u8, Rating)>,
    /// Завершение пачки: штатный advance урока.
    on_next: Callback<()>,
    #[prop(into)] test_id: Signal<String>,
) -> impl IntoView {
    let i18n = use_i18n();
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

    let rate = move |rating: Rating| {
        if !revealed.get_untracked() || finished.get_untracked() {
            return;
        }
        let index = current.get_untracked();
        let Some(item) = items.with_value(|i| i.get(index).cloned()) else {
            return;
        };
        on_rate_binding.run((item.number(), rating));
        if index + 1 >= total {
            // Последняя связка: отвеченная карта ОСТАЁТСЯ на экране
            // (замороженный ответ — юзер-репорт 0.8.0-rc: сброс revealed
            // мелькал фронтом «цифра×суффикс» перед следующей карточкой
            // урока) — advance сразу, без промежуточного фронт-кадра.
            // Повторные 1/2/клики гасятся finished-гейтом выше.
            finished.set(true);
            on_next.run(());
        } else {
            revealed.set(false);
            highlighted_number.set(u8::MAX);
            current.set(index + 1);
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

    let progress = Signal::derive(move || format!("{} / {}", current.get() + 1, total));

    view! {
        <Card class=Signal::derive(|| super::LESSON_CARD_CLASS.to_string()) shadow=true test_id=test_id>
            <div class="flex flex-col gap-4 text-center">
                <p class="font-mono text-[var(--text-label-sm)] uppercase tracking-[0.1em] text-[var(--fg-muted)]"
                   data-testid="counter-bindings-progress">
                    {move || progress.get()}
                </p>

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
                        <Button
                            variant=Signal::derive(|| ButtonVariant::Filled)
                            test_id=Signal::derive(|| "counter-binding-show-answer-btn".to_string())
                            on_click=Callback::new(move |_: leptos::ev::MouseEvent| reveal())
                        >
                            <span>{t!(i18n, lesson.show_answer)}</span>
                            <span class="kbd-hint text-[var(--fg-light)]">{t!(i18n, lesson.space_key)}</span>
                        </Button>
                    </div>
                </Show>

                <Show when=move || revealed.get()>
                    {move || {
                        let number = items.with_value(|i| i.get(current.get()).map(|it| it.number())).unwrap_or(u8::MAX);
                        let reading = readings
                            .with_value(|rs| rs.iter().find(|(n, _)| *n == number).map(|(_, r)| r.clone()))
                            .unwrap_or_default();
                        view! {
                            <div class="flex items-center justify-center gap-2" data-testid="counter-binding-answer">
                                <span class="font-mono text-5xl text-[var(--fg-black)]">
                                    {number_label(number)}
                                </span>
                                <span class="text-3xl text-[var(--fg-muted)]">{"×"}</span>
                                <span class="font-serif text-7xl text-[var(--fg-black)]">
                                    {suffix.with_value(String::clone)}
                                </span>
                            </div>
                            <p class="font-serif text-4xl text-[var(--fg-black)]" data-testid="counter-binding-answer-reading">
                                {reading}
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
                            <div class="flex justify-center">
                                <RatingButtonsView on_rate=on_rate />
                            </div>
                        }
                    }}
                </Show>
            </div>
        </Card>
    }
}
