//! Список счётных суффиксов: реестр по уровням + статус колоды юзера.

use crate::i18n::*;
use crate::ui_components::{Button, ButtonVariant, Tag, TagVariant, Text, TextSize};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::components::A;
use origa::domain::{Card, JapaneseLevel};
use origa::traits::UserRepository;
use origa::use_cases::AddCounterCardsUseCase;

use super::add_modal::CountersAddModal;

/// Строка реестра, обогащённая статусом колоды юзера.
#[derive(Clone, PartialEq)]
struct CounterRow {
    suffix: String,
    gloss: String,
    level: JapaneseLevel,
    in_deck: bool,
    known_bindings: usize,
    total_bindings: usize,
}

#[component]
pub fn CountersContent(refresh_trigger: RwSignal<u32>) -> impl IntoView {
    let i18n = use_i18n();
    let repository =
        use_context::<crate::repository::HybridUserRepository>().expect("repository context");
    let is_add_open = RwSignal::new(false);

    let rows = RwSignal::new(Vec::<CounterRow>::new());
    let selected_level = RwSignal::new(JapaneseLevel::N5);

    let load = move || {
        let repository = repository.clone();
        let rows = rows;
        let disposed = StoredValue::new(());
        spawn_local(async move {
            let Some(user) = repository.get_current_user().await.ok().flatten() else {
                return;
            };
            if disposed.is_disposed() {
                return;
            }
            let lang = *user.native_language();
            let deck: std::collections::HashMap<String, (bool, usize, usize)> = user
                .knowledge_set()
                .study_cards()
                .values()
                .filter_map(|sc| match sc.card() {
                    Card::Counter(counter) => {
                        let total = counter.bindings().len();
                        let known = counter
                            .bindings()
                            .iter()
                            .filter(|b| b.memory().is_known_card())
                            .count();
                        Some((counter.suffix().to_string(), (true, known, total)))
                    },
                    _ => None,
                })
                .collect();

            let mut out: Vec<CounterRow> = origa::dictionary::counters::COUNTERS
                .get()
                .map(|entries| {
                    entries
                        .iter()
                        .map(|entry| {
                            let (in_deck, known_bindings, total_bindings) = deck
                                .get(entry.suffix())
                                .cloned()
                                .unwrap_or((false, 0, entry.readings().len()));
                            CounterRow {
                                suffix: entry.suffix().to_string(),
                                gloss: origa::dictionary::counters::gloss_for(entry, lang)
                                    .to_string(),
                                level: entry.level(),
                                in_deck,
                                known_bindings,
                                total_bindings,
                            }
                        })
                        .collect()
                })
                .unwrap_or_default();
            out.sort_by_key(|r| (r.level, r.suffix.clone()));
            rows.set(out);
        });
    };

    Effect::new(move |_| {
        let _ = refresh_trigger.get();
        load();
    });

    let visible = Memo::new(move |_| {
        let level = selected_level.get();
        rows.get()
            .into_iter()
            .filter(|r| r.level <= level)
            .collect::<Vec<_>>()
    });

    view! {
        <div class="space-y-4">
            <div class="flex items-center justify-between flex-wrap gap-3">
                <Text size=TextSize::Large>{t!(i18n, counters.title)}</Text>
                <Button
                    variant=ButtonVariant::Filled
                    on_click=Callback::new(move |_: leptos::ev::MouseEvent| is_add_open.set(true))
                    test_id="counters-add-open"
                >
                    {t!(i18n, counters.add)}
                </Button>
            </div>

            <div class="flex gap-2 flex-wrap" data-testid="counters-level-filter">
                {(JapaneseLevel::ALL).iter().map(|level| {
                    let is_active = Signal::derive({
                        let level = *level;
                        move || selected_level.get() == level
                    });
                    view! {
                        <button
                            class=move || {
                                let base = "px-3 py-1 border font-mono text-sm";
                                if is_active.get() {
                                    format!("{base} bg-[var(--fg-black)] text-[var(--bg-paper)]")
                                } else {
                                    format!("{base} border-[var(--fg-black)]")
                                }
                            }
                            on:click=move |_: leptos::ev::MouseEvent| selected_level.set(*level)
                        >
                            {level.to_string()}
                        </button>
                    }
                }).collect::<Vec<_>>()}
            </div>

            <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3" data-testid="counters-grid">
                {move || {
                    visible
                        .get()
                        .into_iter()
                        .map(|row| {
                            let CounterRow {
                                suffix,
                                gloss,
                                level,
                                in_deck,
                                known_bindings,
                                total_bindings,
                            } = row;
                            view! {
                                <A href=format!("/counters/{suffix}") attr:class="block border border-[var(--fg-black)] bg-[var(--bg-paper)] hover:bg-[var(--bg-aged)] p-4">
                                    <div class="flex items-start justify-between gap-2">
                                        <span class="font-serif text-4xl text-[var(--fg-black)]"
                                              data-testid="counters-item-suffix">{suffix.clone()}</span>
                                        <div class="flex flex-col items-end gap-1">
                                            <Tag variant=TagVariant::Olive>{level.to_string()}</Tag>
                                            {if in_deck {
                                                view! {
                                                    <span class="text-[var(--text-label-sm)] text-[var(--accent-sage)] font-mono"
                                                          data-testid="counters-item-in-deck">
                                                        {t!(i18n, counters.in_deck)}
                                                    </span>
                                                }.into_any()
                                            } else {
                                                ().into_any()
                                            }}
                                        </div>
                                    </div>
                                    <p class="text-sm text-[var(--fg-muted)] mt-2">{gloss}</p>
                                    {in_deck.then(|| {
                                        view! {
                                            <p class="text-xs font-mono text-[var(--fg-muted)] mt-1"
                                               data-testid="counters-item-bindings">
                                                {move || {
                                                    let label = td_string!(
                                                        i18n.get_locale(),
                                                        counters.bindings_learned
                                                    );
                                                    format!("{label}: {known_bindings}/{total_bindings}")
                                                }}
                                            </p>
                                        }
                                    })}
                                </A>
                            }
                        })
                        .collect::<Vec<_>>()
                }}
            </div>
        </div>

        <CountersAddModal is_open=is_add_open refresh_trigger=refresh_trigger />
    }
}

/// Кнопка заведения на детальной карточке (переиспользует use case).
pub(crate) async fn add_counters(
    repository: &crate::repository::HybridUserRepository,
    suffixes: Vec<String>,
) -> Result<(usize, usize), origa::domain::OrigaError> {
    AddCounterCardsUseCase::new(repository)
        .execute(suffixes)
        .await
}
