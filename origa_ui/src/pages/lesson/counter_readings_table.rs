//! Таблица чтений счётного суффикса (issue #415): единый рендер для слайда
//! показа и ответа тренировки. Числа по возрастанию, 何 последней,
//! нерегулярные строки подсвечены.

use leptos::prelude::*;

/// Строка таблицы чтений для рендера: контент резолвится из реестра.
#[derive(Clone)]
pub(in crate::pages::lesson) struct CounterReadingRow {
    pub number: u8,
    pub number_label: String,
    pub reading: String,
    pub irregular: bool,
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

/// Таблица чтений суффикса: числа по возрастанию, 何 последней,
/// нерегулярные строки подсвечены.
#[component]
pub(in crate::pages::lesson) fn CounterReadingsTable(
    rows: Vec<CounterReadingRow>,
    suffix: String,
    #[prop(into)] test_id: Signal<String>,
    /// Акцент строки текущей связки пачки (u8::MAX — вне диапазона).
    #[prop(optional)]
    highlight: Option<RwSignal<u8>>,
) -> impl IntoView {
    let suffix = StoredValue::new(suffix);
    // Сортировка — инвариант самого компонента (любой источник строк):
    // числа по возрастанию, исключения >10 за десяткой, 何 последней.
    let mut sorted_rows = rows;
    sorted_rows.sort_by_key(|row| reading_sort_key(row.number));
    let rows_vec = sorted_rows;
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
