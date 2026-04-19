use super::super::shared::{
    CardCounts, CardStatus, Filter, FilterBtn, LoadMoreButton, create_delete_callback,
    create_mark_as_known_callback,
};
use super::phrase_card_item::PhraseCardItem;
use crate::i18n::{t, use_i18n};
use crate::repository::HybridUserRepository;
use crate::ui_components::{
    Input, LoadingOverlay, Text, TextSize, ToastContainer, ToastData, TypographyVariant,
};
use leptos::either::Either;
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::domain::{Card, StudyCard, User};
use origa::traits::UserRepository;
use origa::use_cases::ToggleFavoriteUseCase;
use ulid::Ulid;

#[component]
pub fn PhrasesContent(refresh_trigger: RwSignal<u32>) -> impl IntoView {
    let i18n = use_i18n();
    let repository =
        use_context::<HybridUserRepository>().expect("repository context not provided");

    let current_user: RwSignal<Option<User>> = RwSignal::new(None);
    let is_loading = RwSignal::new(true);
    let all_cards: RwSignal<Vec<StudyCard>> = RwSignal::new(Vec::new());
    let repo_for_effect = repository.clone();

    Effect::new(move |_| {
        let _ = refresh_trigger.get();
        let repo = repo_for_effect.clone();
        let disposed = StoredValue::new(());
        spawn_local(async move {
            match repo.get_current_user().await {
                Ok(Some(user)) => {
                    if disposed.is_disposed() {
                        return;
                    }
                    let cards = user
                        .knowledge_set()
                        .study_cards()
                        .iter()
                        .filter(|(_, card)| matches!(card.card(), Card::Phrase(_)))
                        .map(|(_, card)| card.clone())
                        .collect();
                    all_cards.set(cards);
                    current_user.set(Some(user));
                    is_loading.set(false);
                },
                Ok(None) => {
                    tracing::warn!("PhrasesContent: user not found");
                },
                Err(e) => {
                    tracing::error!("PhrasesContent: get_current_user error: {:?}", e);
                },
            }
        });
    });

    let native_lang =
        Memo::new(move |_| crate::i18n::locale_to_native_language(&i18n.get_locale()));

    let known_kanji = Memo::new(move |_| {
        current_user
            .get()
            .map(|u| u.knowledge_set().get_known_kanji())
            .unwrap_or_default()
    });

    let search = RwSignal::new(String::new());
    let filter = RwSignal::new(Filter::All);
    let toasts: RwSignal<Vec<ToastData>> = RwSignal::new(Vec::new());
    let visible_count: RwSignal<usize> = RwSignal::new(50);

    let on_toggle_favorite = {
        let repo = repository.clone();
        Callback::new(move |card_id: Ulid| {
            let repository = repo.clone();
            let user_signal = current_user;
            let disposed = StoredValue::new(());
            spawn_local(async move {
                let use_case = ToggleFavoriteUseCase::new(&repository);
                if use_case.execute(card_id).await.is_ok() {
                    if disposed.is_disposed() {
                        return;
                    }
                    user_signal.update(|u| {
                        if let Some(user) = u {
                            let _ = user.toggle_favorite(card_id);
                        }
                    });
                }
            });
        })
    };

    let on_mark_as_known = create_mark_as_known_callback(repository.clone(), refresh_trigger);
    let (is_deleting, on_delete) =
        create_delete_callback(repository.clone(), toasts, refresh_trigger);

    let filtered_cards = Memo::new(move |_| {
        let query = search.get().to_lowercase();
        let current_filter = filter.get();
        let lang = native_lang.get();

        all_cards
            .get()
            .into_iter()
            .filter(|card| {
                let matches_search = query.is_empty() || {
                    let card_inner = card.card();
                    let question = card_inner.question(&lang);
                    let answer = card_inner.answer(&lang);

                    if let Ok(question) = question
                        && let Ok(answer) = answer
                    {
                        question.text().to_lowercase().contains(&query)
                            || answer.text().to_lowercase().contains(&query)
                    } else {
                        false
                    }
                };
                let matches_filter = current_filter.matches(CardStatus::from_study_card(card));
                matches_search && matches_filter
            })
            .collect::<Vec<_>>()
    });

    Effect::new(move |_| {
        let _ = search.get();
        let _ = filter.get();
        let _ = all_cards.get();
        visible_count.set(50);
    });

    let visible_cards = Memo::new(move |_| {
        filtered_cards
            .get()
            .into_iter()
            .take(visible_count.get())
            .collect::<Vec<_>>()
    });

    let counts = Memo::new(move |_| {
        let cards = all_cards.get();
        cards.iter().fold(CardCounts::default(), |mut acc, card| {
            acc.total += 1;
            match CardStatus::from_study_card(card) {
                CardStatus::New => acc.new += 1,
                CardStatus::Hard => acc.hard += 1,
                CardStatus::InProgress => acc.in_progress += 1,
                CardStatus::Learned => acc.learned += 1,
            }
            acc
        })
    });

    view! {
        <div class="space-y-4">
            <Show when=move || is_loading.get()>
                <LoadingOverlay message=Signal::derive(move || i18n.get_keys().common().loading().inner().to_string()) />
            </Show>
            <Show when=move || !is_loading.get()>
                <Input
                    value=search
                    placeholder=Signal::derive(move || i18n.get_keys().common().search().inner().to_string())
                    test_id="phrases-search-input"
                />

                <div class="flex flex-wrap gap-2">
                    <FilterBtn filter=Filter::All count=move || counts.get().total active=filter test_id="phrases-filter-all" />
                    <FilterBtn filter=Filter::New count=move || counts.get().new active=filter test_id="phrases-filter-new" />
                    <FilterBtn filter=Filter::Hard count=move || counts.get().hard active=filter test_id="phrases-filter-hard" />
                    <FilterBtn filter=Filter::InProgress count=move || counts.get().in_progress active=filter test_id="phrases-filter-in-progress" />
                    <FilterBtn filter=Filter::Learned count=move || counts.get().learned active=filter test_id="phrases-filter-learned" />
                </div>

                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 2xl:grid-cols-5 gap-4" data-testid="phrases-grid">
                    {move || {
                        let cards = visible_cards.get();
                        if cards.is_empty() {
                            Either::Left(view! {
                                <div class="col-span-full" data-testid="phrases-empty-state">
                                    <Text size=TextSize::Default variant=TypographyVariant::Muted>
                                        {t!(i18n, phrases.not_found)}
                                    </Text>
                                </div>
                            })
                        } else {
                            Either::Right(view! {
                                <For
                                    each=move || visible_cards.get()
                                    key=|card| *card.card_id()
                                    children=move |card| {
                                        let card_id = *card.card_id();
                                        view! {
                                            <PhraseCardItem
                                                study_card=card
                                                native_language=native_lang
                                                known_kanji=known_kanji.get()
                                                on_toggle_favorite=on_toggle_favorite
                                                on_mark_as_known=Callback::new(move |_| on_mark_as_known.run(card_id))
                                                on_delete=on_delete
                                                is_deleting=is_deleting.into()
                                            />
                                        }
                                    }
                                />
                            })
                        }
                    }}
                </div>
                <LoadMoreButton
                    visible_count=visible_count
                    total=Signal::derive(move || filtered_cards.get().len())
                    test_id=Signal::derive(|| "phrases-load-more-btn".to_string())
                />
                <ToastContainer toasts=toasts duration_ms=5000 />
            </Show>
        </div>
    }
}
