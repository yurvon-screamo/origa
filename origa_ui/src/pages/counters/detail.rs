//! Детальная карточка счётного суффикса (`/counters/:suffix`): знак, глосса,
//! уровень, таблица чтений с состоянием каждой связки, заведение/лайк/удаление.

use crate::i18n::*;
use crate::ui_components::{Button, ButtonVariant, Tag, TagVariant, Text, TextSize};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_params;
use leptos_router::params::Params;
use origa::domain::{Card, JapaneseLevel};
use origa::traits::UserRepository;
use origa::use_cases::{DeleteCardUseCase, MarkCardAsKnownUseCase, ToggleFavoriteUseCase};

#[derive(Params, PartialEq, Clone)]
struct CounterParams {
    suffix: String,
}

#[derive(Clone)]
struct DetailState {
    suffix: String,
    gloss: String,
    level: JapaneseLevel,
    deck_card: Option<origa::domain::StudyCard>,
}

#[component]
pub fn CountersDetail() -> impl IntoView {
    let i18n = use_i18n();
    let repository =
        use_context::<crate::repository::HybridUserRepository>().expect("repository context");
    let params = use_params::<CounterParams>();
    let refresh = RwSignal::new(0u32);
    let state = RwSignal::new(None::<DetailState>);
    let not_found = RwSignal::new(false);

    Effect::new({
        let repository = repository.clone();
        move |_| {
            let _ = refresh.get();
            let Ok(p) = params.get() else { return };
            let suffix = p.suffix;
            let repository = repository.clone();
            let state = state;
            let not_found = not_found;
            spawn_local(async move {
                let Some(entry) = origa::dictionary::counters::get_counter(&suffix) else {
                    not_found.set(true);
                    state.set(None);
                    return;
                };
                not_found.set(false);
                let user = repository.get_current_user().await.ok().flatten();
                let lang = user
                    .as_ref()
                    .map(|u| *u.native_language())
                    .unwrap_or(origa::domain::NativeLanguage::English);
                let deck_card = user.as_ref().and_then(|u| {
                    u.knowledge_set()
                        .study_cards()
                        .values()
                        .find_map(|sc| match sc.card() {
                            Card::Counter(c) if c.suffix() == suffix => Some(sc.clone()),
                            _ => None,
                        })
                });
                state.set(Some(DetailState {
                    suffix: suffix.clone(),
                    gloss: origa::dictionary::counters::gloss_for(entry, lang).to_string(),
                    level: entry.level(),
                    deck_card,
                }));
            });
        }
    });

    let run = Callback::new({
        let repository = repository.clone();
        move |action: Action| {
            let repository = repository.clone();
            spawn_local(async move {
                let result = match action {
                    Action::Add(suffix) => super::content::add_counters(&repository, vec![suffix])
                        .await
                        .map(|_| ()),
                    Action::Delete(card_id) => {
                        DeleteCardUseCase::new(&repository).execute(card_id).await
                    },
                    Action::Favorite(card_id) => ToggleFavoriteUseCase::new(&repository)
                        .execute(card_id)
                        .await
                        .map(|_| ()),
                    Action::MarkKnown(card_id) => {
                        MarkCardAsKnownUseCase::new(&repository)
                            .execute(card_id)
                            .await
                    },
                };
                if let Err(e) = result {
                    tracing::error!("Counters detail action failed: {e}");
                }
                refresh.update(|t| *t += 1);
            });
        }
    });

    let back = Callback::new(move |_: leptos::ev::MouseEvent| {
        let nav = leptos_router::hooks::use_navigate();
        nav("/counters", Default::default());
    });

    view! {
        <div class="space-y-4 max-w-2xl" data-testid="counters-detail">
            <Show
                when=move || !not_found.get()
                fallback=move || {
                    view! {
                        <div class="space-y-4">
                            <Text size=TextSize::Large>
                                {move || td_string!(i18n.get_locale(), counters.not_found).to_string()}
                            </Text>
                            <Button variant=ButtonVariant::Ghost on_click=back test_id="counters-detail-back-error">
                                {t!(i18n, counters.back)}
                            </Button>
                        </div>
                    }
                }
            >
                {move || {
                    let Some(st) = state.get() else {
                        return ().into_any();
                    };
                    let rows = reading_rows(&st);
                    let known_count = rows.iter().filter(|r| r.2).count();
                    view! {
                        <div class="space-y-4">
                            <div class="flex items-center justify-between">
                                <Button variant=ButtonVariant::Ghost on_click=back test_id="counters-detail-back">
                                    {t!(i18n, counters.back)}
                                </Button>
                                <Tag variant=TagVariant::Olive>{st.level.to_string()}</Tag>
                            </div>

                            <div class="text-center py-4" data-testid="counters-detail-head">
                                <p class="font-serif text-7xl text-[var(--fg-black)]">{st.suffix.clone()}</p>
                                <p class="font-mono text-[var(--fg-muted)] mt-2">{st.gloss.clone()}</p>
                                {st.deck_card.is_some().then(|| {
                                    let label = td_string!(i18n.get_locale(), counters.bindings_learned);
                                    view! {
                                        <p class="text-xs font-mono text-[var(--fg-muted)] mt-1"
                                           data-testid="counters-detail-bindings">
                                            {format!("{label}: {known_count}/{}", rows.len())}
                                        </p>
                                    }
                                })}
                            </div>

                            {match &st.deck_card {
                                None => {
                                    view! {
                                        <div class="flex justify-center">
                                            <Button
                                                variant=ButtonVariant::Filled
                                                on_click=Callback::new({
                                                    let run = run;
                                                    let suffix = st.suffix.clone();
                                                    move |_: leptos::ev::MouseEvent| run.run(Action::Add(suffix.clone()))
                                                })
                                                test_id="counters-detail-add"
                                            >
                                                {t!(i18n, counters.add)}
                                            </Button>
                                        </div>
                                    }.into_any()
                                },
                                Some(sc) => {
                                    let card_id = *sc.card_id();
                                    let is_favorite = sc.is_favorite();
                                    view! {
                                        <div class="flex justify-center gap-2 flex-wrap" data-testid="counters-detail-actions">
                                            <Button
                                                variant=ButtonVariant::Ghost
                                                on_click=Callback::new({
                                                    let run = run;
                                                    move |_: leptos::ev::MouseEvent| run.run(Action::MarkKnown(card_id))
                                                })
                                                test_id="counters-detail-known"
                                            >
                                                {t!(i18n, counters.mark_known)}
                                            </Button>
                                            <Button
                                                variant=ButtonVariant::Ghost
                                                on_click=Callback::new({
                                                    let run = run;
                                                    move |_: leptos::ev::MouseEvent| run.run(Action::Favorite(card_id))
                                                })
                                                test_id="counters-detail-favorite"
                                            >
                                                {if is_favorite { td_string!(i18n.get_locale(), counters.unfavorite) } else { td_string!(i18n.get_locale(), counters.favorite) }}
                                            </Button>
                                            <Button
                                                variant=ButtonVariant::Ghost
                                                on_click=Callback::new({
                                                    let run = run;
                                                    move |_: leptos::ev::MouseEvent| run.run(Action::Delete(card_id))
                                                })
                                                test_id="counters-detail-delete"
                                            >
                                                {t!(i18n, counters.delete)}
                                            </Button>
                                        </div>
                                    }.into_any()
                                },
                            }}

                            <div class="border border-[var(--fg-black)] bg-[var(--bg-paper)] overflow-hidden"
                                 data-testid="counters-detail-table">
                                {rows
                                    .into_iter()
                                    .map(|(label, reading, known)| {
                                        view! {
                                            <div class=format!(
                                                "flex justify-between px-4 py-1.5 border-b border-[var(--fg-light)] last:border-b-0 {}",
                                                if known { "bg-[var(--accent-sage)]/30" } else { "" },
                                            )>
                                                <span class="font-mono text-[var(--fg-black)]">
                                                    {label}{"×"}{st.suffix.clone()}
                                                </span>
                                                <span class="font-serif text-[var(--fg-black)]">{reading}</span>
                                            </div>
                                        }
                                    })
                                    .collect::<Vec<_>>()}
                            </div>
                        </div>
                    }
                    .into_any()
                }}
            </Show>
        </div>
    }
}

/// Действия детальной карточки.
#[derive(Clone)]
enum Action {
    Add(String),
    Delete(ulid::Ulid),
    Favorite(ulid::Ulid),
    MarkKnown(ulid::Ulid),
}

/// Строки таблицы: (номер-лейбл, чтение, изучена). Контент — реестр,
/// статус связок — из карты юзера.
fn reading_rows(st: &DetailState) -> Vec<(String, String, bool)> {
    let Some(entry) = origa::dictionary::counters::get_counter(&st.suffix) else {
        return Vec::new();
    };
    let binding_known = |number: u8| -> bool {
        st.deck_card.as_ref().is_some_and(|sc| match sc.card() {
            Card::Counter(counter) => counter
                .binding_memory(number)
                .is_some_and(|m| m.is_known_card()),
            _ => false,
        })
    };
    entry
        .readings()
        .iter()
        .map(|r| {
            (
                match r.number() {
                    0 => "何".to_string(),
                    n => n.to_string(),
                },
                r.reading().to_string(),
                binding_known(r.number()),
            )
        })
        .collect()
}
