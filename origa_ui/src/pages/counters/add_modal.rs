//! Модалка заведения счётных суффиксов (issue #415): чекбоксы по уровню,
//! «добавить выбранные» — AddCounterCardsUseCase.

use crate::i18n::*;
use crate::ui_components::{Button, ButtonVariant, Modal, Tag, TagVariant, Text, TextSize};
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::dictionary::counters::COUNTERS;
use origa::domain::JapaneseLevel;

#[component]
pub fn CountersAddModal(is_open: RwSignal<bool>, refresh_trigger: RwSignal<u32>) -> impl IntoView {
    let i18n = use_i18n();
    let repository =
        use_context::<crate::repository::HybridUserRepository>().expect("repository context");
    let selected_level = RwSignal::new(JapaneseLevel::N5);
    let selected: RwSignal<std::collections::HashSet<String>> =
        RwSignal::new(std::collections::HashSet::new());
    let is_creating = RwSignal::new(false);

    // Сброс выбора при открытии.
    Effect::new(move |_| {
        if is_open.get() {
            selected.set(std::collections::HashSet::new());
        }
    });

    let candidates = Memo::new(move |_| {
        let level = selected_level.get();
        COUNTERS
            .get()
            .map(|entries| {
                entries
                    .iter()
                    .filter(|e| e.level() <= level)
                    .map(|e| (e.suffix().to_string(), e.level()))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    });

    let on_add = Callback::new(move |_: leptos::ev::MouseEvent| {
        let suffixes: Vec<String> = selected.get_untracked().into_iter().collect();
        if suffixes.is_empty() || is_creating.get_untracked() {
            return;
        }
        is_creating.set(true);
        let repository = repository.clone();
        spawn_local(async move {
            match super::content::add_counters(&repository, suffixes).await {
                Ok((created, skipped)) => {
                    tracing::info!(created, skipped, "Counters page: cards added");
                    refresh_trigger.update(|t| *t += 1);
                    is_open.set(false);
                },
                Err(e) => tracing::error!("Counters page: add failed: {e}"),
            }
            is_creating.set(false);
        });
    });

    view! {
        <Modal is_open title=Signal::derive(move || t_string_shim(&i18n)) test_id=Signal::derive(|| "counters-add-modal".to_string())>
            <div class="space-y-4">
                <Text size=TextSize::Large>{t!(i18n, counters.add_title)}</Text>

                <div class="flex gap-2 flex-wrap" data-testid="counters-add-level">
                    {(JapaneseLevel::ALL).iter().map(|level| {
                        let is_active = Signal::derive({
                            let level = *level;
                            move || selected_level.get() == level
                        });
                        view! {
                            <button
                                class=move || {
                                    let base = "px-3 py-1 border font-mono text-sm";
                                    if is_active.get() { format!("{base} bg-[var(--fg-black)] text-[var(--bg-paper)]") } else { format!("{base} border-[var(--fg-black)]") }
                                }
                                on:click=move |_| selected_level.set(*level)
                            >
                                {level.to_string()}
                            </button>
                        }
                    }).collect::<Vec<_>>()}
                </div>

                <div class="grid grid-cols-2 sm:grid-cols-3 gap-2 max-h-80 overflow-y-auto" data-testid="counters-add-list">
                    {move || {
                        candidates
                            .get()
                            .into_iter()
                            .map(|(suffix, level)| {
                                let is_checked = Signal::derive({
                                    let suffix = suffix.clone();
                                    move || selected.get().contains(&suffix)
                                });
                                let suffix_click = suffix.clone();
                                view! {
                                    <button
                                        class=move || {
                                            let base = "flex items-center justify-between border px-3 py-2 text-left";
                                            if is_checked.get() {
                                                format!("{base} bg-[var(--accent-olive)] border-[var(--fg-black)]")
                                            } else {
                                                format!("{base} border-[var(--fg-light)]")
                                            }
                                        }
                                        on:click=move |_| {
                                            selected.update(|s| {
                                                if !s.insert(suffix_click.clone()) {
                                                    s.remove(&suffix_click);
                                                }
                                            });
                                        }
                                    >
                                        <span class="font-serif text-2xl">{suffix}</span>
                                        <Tag variant=TagVariant::Olive>{level.to_string()}</Tag>
                                    </button>
                                }
                            })
                            .collect::<Vec<_>>()
                    }}
                </div>

                <div class="flex justify-end gap-2">
                    <Button variant=ButtonVariant::Ghost on_click=Callback::new(move |_: leptos::ev::MouseEvent| is_open.set(false)) test_id="counters-add-cancel">
                        {t!(i18n, common.cancel)}
                    </Button>
                    <Button
                        variant=ButtonVariant::Filled
                        on_click=on_add
                        test_id="counters-add-confirm"
                        disabled=Signal::derive(move || is_creating.get())
                    >
                        {t!(i18n, counters.add_confirm)}
                    </Button>
                </div>
            </div>
        </Modal>
    }
}

/// Заголовок модалки — обёртка над t! для Signal::derive.
fn t_string_shim(i18n: &I18nContext<Locale>) -> String {
    td_string!(i18n.get_locale_untracked(), counters.add_title).to_string()
}
