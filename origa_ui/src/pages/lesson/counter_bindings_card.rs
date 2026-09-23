//! Композитный слот счётного суффикса (issue #415): пачка мини-вопросов
//! «число × суффикс → выбор чтения» в одном слоте урока. Каждый ответ
//! рейтингует ТОЛЬКО память своей связки (`RateCounterBindingUseCase`,
//! CounterReview); завершение пачки — агрегат через штатный `on_rate`
//! (Good / любая ошибка → Again): память семантики в StandardLesson,
//! одна дневная запись, бюджет новичков один раз.
//!
//! После каждого ответа — таблица мутаций (все связки по возрастанию
//! числа, 何 последней; нерегулярные подсвечены).

use crate::i18n::*;
use leptos::prelude::*;
use origa::domain::{Card, CounterBindingPrompt, Rating};
use origa::traits::UserRepository;

/// Строка таблицы мутаций для рендера: контент резолвится из реестра.
#[derive(Clone)]
pub(in crate::pages::lesson) struct CounterReadingRow {
    pub number: u8,
    pub number_label: String,
    pub reading: String,
    pub irregular: bool,
}

fn number_label(number: u8) -> String {
    match number {
        0 => "何".to_string(),
        n => n.to_string(),
    }
}

/// Порядок таблицы: числа по возрастанию, лексикализованные исключения
/// >10 за десяткой, вопросительное 何 последним (инвариант датасета).
fn reading_sort_key(number: u8) -> (u8, u8) {
    match number {
        0 => (2, 0),
        n if (1..=10).contains(&n) => (0, n),
        n => (1, n),
    }
}

pub(in crate::pages::lesson) fn counter_reading_rows(card: &Card) -> Vec<CounterReadingRow> {
    let Card::Counter(counter) = card else {
        return Vec::new();
    };
    let entry = origa::dictionary::counters::get_counter(counter.suffix());
    let mut rows: Vec<CounterReadingRow> = counter
        .bindings()
        .iter()
        .filter_map(|binding| {
            let reading = entry
                .and_then(|e| e.reading_for(binding.number()))
                .map(str::to_string)
                .unwrap_or_default();
            let irregular = entry
                .map(|e| e.irregular_for(binding.number()))
                .unwrap_or(false);
            (!reading.is_empty()).then_some(CounterReadingRow {
                number: binding.number(),
                number_label: number_label(binding.number()),
                reading,
                irregular,
            })
        })
        .collect();
    rows.sort_by_key(|row| reading_sort_key(row.number));
    rows
}

/// Таблица чтений суффикса — единый рендер трёх поверхностей
/// (композитный слот, слайд показа, ответ тренировки). Числа по
/// возрастанию, 何 последней, нерегулярные строки подсвечены;
/// `highlight` акцентирует строку только что отвеченной связки.
#[component]
pub(in crate::pages::lesson) fn CounterReadingsTable(
    rows: Vec<CounterReadingRow>,
    suffix: String,
    #[prop(into)] test_id: Signal<String>,
    #[prop(optional)] highlight: Option<RwSignal<u8>>,
) -> impl IntoView {
    let suffix = StoredValue::new(suffix);
    let rows = StoredValue::new(rows);
    let rows_vec = rows.with_value(|rows| rows.to_vec());
    view! {
        <div class="border border-[var(--fg-black)] bg-[var(--bg-paper)] overflow-hidden"
             data-testid=test_id>
            {rows_vec
                .into_iter()
                .map(|row| {
                    let number = row.number;
                    let base = "flex justify-between px-4 py-1.5 border-b border-[var(--fg-light)] last:border-b-0";
                    view! {
                        <div class=move || {
                            let mut class = base.to_string();
                            if row.irregular {
                                class.push_str(" bg-[var(--accent-warm)]");
                            }
                            if highlight.is_some_and(|h| h.get() == number) {
                                class.push_str(" ring-2 ring-inset ring-[var(--accent-olive)]");
                            }
                            class
                        }
                             data-testid="counter-mutations-row">
                            <span class="font-mono text-[var(--fg-black)]">
                                {row.number_label.clone()}{"×"}{suffix.with_value(String::clone)}
                            </span>
                            <span class="font-serif text-[var(--fg-black)]">{row.reading.clone()}</span>
                        </div>
                    }
                })
                .collect::<Vec<_>>()}
        </div>
    }
}

#[component]
pub fn CounterBindingsCard(
    card: Card,
    items: Vec<CounterBindingPrompt>,
    /// Мини-оценка связки: (number, rating) — вызывающий подключает
    /// `RateCounterBindingUseCase` со своим репозиторием.
    on_rate_binding: Callback<(u8, Rating)>,
    /// Агрегат пачки: штатный рейтинговый путь урока (advance + запись
    /// истории + бюджет семантики).
    on_rate: Callback<Rating>,
    #[prop(into)] test_id: Signal<String>,
) -> impl IntoView {
    let i18n = use_i18n();
    let current = RwSignal::new(0usize);
    let selected: RwSignal<Option<String>> = RwSignal::new(None);
    let answered = RwSignal::new(false);
    let correct_count = RwSignal::new(0usize);
    let mistakes = RwSignal::new(false);
    let finished = RwSignal::new(false);
    // StoredValue (Copy): Fn-замыкания Show получают копию хендла, не
    // владея данными (паттерн translator.rs).
    let items = StoredValue::new(items);
    let total = items.with_value(Vec::len);
    let suffix = StoredValue::new(match &card {
        Card::Counter(counter) => counter.suffix().to_string(),
        _ => String::new(),
    });
    // Акцент строки текущего мини-вопроса: таблица мутаций после ответа
    // подсвечивает отвеченную связку (u8::MAX — вне диапазона чисел).
    let highlighted_number = RwSignal::new(u8::MAX);

    let on_select = move |answer: String| {
        if answered.get_untracked() {
            return;
        }
        let index = current.get_untracked();
        let Some(prompt) = items.with_value(|i| i.get(index).cloned()) else {
            return;
        };
        let correct = prompt.check_answer(&answer);
        selected.set(Some(answer));
        answered.set(true);
        highlighted_number.set(prompt.number());
        if correct {
            correct_count.update(|n| *n += 1);
        } else {
            mistakes.set(true);
        }
        // Мини-оценка: только память этой связки (CounterReview).
        let rating = if correct { Rating::Good } else { Rating::Again };
        on_rate_binding.run((prompt.number(), rating));
    };

    let advance = move || {
        let next = current.get_untracked() + 1;
        if next >= total {
            finished.set(true);
            let rating = if mistakes.get_untracked() {
                Rating::Again
            } else {
                Rating::Good
            };
            on_rate.run(rating);
        } else {
            current.set(next);
            selected.set(None);
            answered.set(false);
        }
    };

    // Клавиатура композитной сессии (единый паттерн урока): [1..4] —
    // выбор варианта, Space — «Дальше» после ответа. Живёт в компоненте
    // (use_event_listener с авто-cleanup на смене слота).
    {
        use leptos_use::use_event_listener;
        let items_kb = items;
        let on_select_kb = { Callback::new(move |answer: String| on_select(answer)) };
        let advance_kb = { Callback::new(move |()| advance()) };
        let _ = use_event_listener(
            document(),
            leptos::ev::keydown,
            move |ev: leptos::ev::KeyboardEvent| {
                let answered_now = answered.get_untracked();
                let key = ev.key();
                if !answered_now && (key == "1" || key == "2" || key == "3" || key == "4") {
                    let index: usize = key.parse().unwrap_or(1) - 1;
                    if let Some(answer) = items_kb.with_value(|items| {
                        items.get(current.get_untracked()).and_then(|prompt| {
                            prompt.options().get(index).map(|o| o.text().to_string())
                        })
                    }) {
                        ev.prevent_default();
                        on_select_kb.run(answer);
                    }
                } else if answered_now && (key == " " || key == "Spacebar") {
                    ev.prevent_default();
                    advance_kb.run(());
                }
            },
        );
    }

    view! {
        <div class="flex flex-col gap-4 w-full" data-testid=test_id>
            <div class="text-center" data-testid="counter-bindings-progress">
                <span class="font-mono text-sm text-[var(--fg-muted)]">
                    {move || format!("{} / {}", current.get() + 1, total)}
                </span>
            </div>

            <Show when=move || !finished.get()>
                {move || {
                    let index = current.get();
                    let Some(prompt) = items.with_value(|i| i.get(index).cloned()) else {
                        return ().into_any();
                    };
                    let suffix = suffix.with_value(String::clone);
                    let question_word = format!("{}{}", number_label(prompt.number()), suffix);
                    let options: Vec<(String, bool)> = prompt
                        .options()
                        .iter()
                        .map(|o| (o.text().to_string(), o.is_correct()))
                        .collect();
                    let answered_now = answered.get();
                    let selected_now = selected.get();

                    view! {
                        <div class="flex flex-col gap-4">
                            <p class="font-serif text-5xl sm:text-6xl text-center text-[var(--fg-black)]"
                               data-testid="counter-binding-question">
                                {question_word}
                            </p>
                            <div class="grid grid-cols-2 gap-2 sm:gap-3">
                                {options
                                    .into_iter()
                                    .enumerate()
                                    .map(|(i, (text, is_correct)): (usize, (String, bool))| {
                                        let is_selected = selected_now.as_deref() == Some(text.as_str());
                                        let base = "p-2 sm:p-4 border text-left transition-all cursor-pointer relative flex flex-col justify-center min-h-[4rem]";
                                        let class = if answered_now {
                                            if is_correct {
                                                format!("{base} quiz-option-correct")
                                            } else if is_selected {
                                                format!("{base} quiz-option-incorrect anima-shake")
                                            } else {
                                                format!("{base} quiz-option-dimmed")
                                            }
                                        } else if is_selected {
                                            format!("{base} quiz-option-neutral ring-2 ring-[var(--accent-olive)]")
                                        } else {
                                            format!("{base} quiz-option-neutral")
                                        };
                                        let text_for_click = text.clone();
                                        view! {
                                            <button
                                                class=class
                                                data-testid=format!("counter-binding-option-{}", i)
                                                on:click=move |_| {
                                                    on_select(text_for_click.clone());
                                                }
                                            >
                                                <span class="font-serif text-xl text-[var(--fg-black)]">{text.clone()}</span>
                                            </button>
                                        }
                                    })
                                    .collect::<Vec<_>>()}
                            </div>
                            <Show when=move || answered_now>
                                <button
                                    class="px-6 py-3 border border-[var(--fg-black)] bg-[var(--accent-olive)] text-[var(--fg-black)] font-mono cursor-pointer"
                                    data-testid="counter-binding-next"
                                    on:click=move |_| advance()
                                >
                                    {t!(i18n, lesson.next)}
                                </button>
                            </Show>
                        </div>
                    }
                    .into_any()
                }}
            </Show>

            // Таблица мутаций: числа по возрастанию, 何 последней,
            // нерегулярные строки подсвечены, отвеченная — акцентирована.
            <CounterReadingsTable
                rows=counter_reading_rows(&card)
                suffix=suffix.with_value(String::clone)
                highlight=highlighted_number
                test_id=Signal::derive(|| "counter-mutations-table".to_string())
            />
            <div class="text-center text-xs font-mono text-[var(--fg-muted)]">
                {move || {
                    let label = td_string!(i18n.get_locale(), lesson.counter_correct);
                    format!("{label}: {}", correct_count.get())
                }}
            </div>
        </div>
    }
}

/// Резолв card_id counter-карты по суффиксу и мини-рейтинг связки:
/// только память ячейки (CounterReview), мимо rate_card.
pub(in crate::pages::lesson) async fn rate_binding_by_suffix<R: UserRepository>(
    repository: &R,
    suffix: &str,
    number: u8,
    rating: Rating,
) -> Result<(), origa::domain::OrigaError> {
    let mut user = repository
        .get_current_user()
        .await?
        .ok_or(origa::domain::OrigaError::CurrentUserNotExist)?;
    let card_id = *user
        .knowledge_set()
        .study_cards()
        .values()
        .find(|sc| matches!(sc.card(), Card::Counter(c) if c.suffix() == suffix))
        .ok_or(origa::domain::OrigaError::CounterBindingNotFound {
            suffix: suffix.to_string(),
            number,
        })?
        .card_id();

    user.rate_counter_binding(card_id, number, rating)?;
    repository.save(&user).await
}
