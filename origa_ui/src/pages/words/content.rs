use super::super::shared::{
    CardCounts, CardStatus, Filter, FilterBtn, create_delete_callback,
    create_mark_as_known_callback,
};
use super::vocabulary_card_item::VocabularyCardItem;
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
pub fn WordsContent(refresh_trigger: RwSignal<u32>) -> impl IntoView {
    let i18n = use_i18n();
    let repository =
        use_context::<HybridUserRepository>().expect("repository context not provided");

    let current_user: RwSignal<Option<User>> = RwSignal::new(None);
    let is_loading = RwSignal::new(true);
    let all_cards: RwSignal<Vec<StudyCard>> = RwSignal::new(Vec::new());
    let repo_for_effect = repository.clone();
    let disposed = StoredValue::new(());

    Effect::new(move |_| {
        let _ = refresh_trigger.get();
        let repo = repo_for_effect.clone();
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
                        .filter(|(_, card)| matches!(card.card(), Card::Vocabulary(_)))
                        .map(|(_, card)| card.clone())
                        .collect();
                    all_cards.set(cards);
                    current_user.set(Some(user));
                    is_loading.set(false);
                },
                Ok(None) => {
                    tracing::warn!("WordsContent: user not found");
                },
                Err(e) => {
                    tracing::error!("WordsContent: get_current_user error: {:?}", e);
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

    let on_toggle_favorite = {
        let repo = repository.clone();

        Callback::new(move |card_id: Ulid| {
            let repository = repo.clone();
            let user_signal = current_user;
            let disposed = disposed;

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
                    let word = card
                        .card()
                        .question(&lang)
                        .ok()
                        .map(|q| q.text().to_string())
                        .unwrap_or_default()
                        .to_lowercase();
                    let meaning = card
                        .card()
                        .answer(&lang)
                        .ok()
                        .map(|a| a.text().to_string())
                        .unwrap_or_default()
                        .to_lowercase();
                    word.contains(&query) || meaning.contains(&query)
                };
                let matches_filter = current_filter.matches(CardStatus::from_study_card(card));
                matches_search && matches_filter
            })
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
                    test_id="words-search-input"
                />

                <div class="flex flex-wrap gap-2">
                    <FilterBtn filter=Filter::All count=move || counts.get().total active=filter test_id="words-filter-all" />
                    <FilterBtn filter=Filter::New count=move || counts.get().new active=filter test_id="words-filter-new" />
                    <FilterBtn filter=Filter::Hard count=move || counts.get().hard active=filter test_id="words-filter-hard" />
                    <FilterBtn filter=Filter::InProgress count=move || counts.get().in_progress active=filter test_id="words-filter-in-progress" />
                    <FilterBtn filter=Filter::Learned count=move || counts.get().learned active=filter test_id="words-filter-learned" />
                </div>

                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 2xl:grid-cols-5 gap-4" data-testid="words-grid">
                    {move || {
                        let cards = filtered_cards.get();
                        if cards.is_empty() {
                            Either::Left(view! {
                                <div class="col-span-full" data-testid="words-empty-state">
                                    <Text size=TextSize::Default variant=TypographyVariant::Muted>
                                        {t!(i18n, words.words_not_found)}
                                    </Text>
                                </div>
                            })
                        } else {
                            Either::Right(view! {
                                <For
                                    each=move || filtered_cards.get()
                                    key=|card| *card.card_id()
                                    children=move |card| {
                                        let card_id = *card.card_id();
                                        view! {
                                            <VocabularyCardItem
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
                <ToastContainer toasts=toasts duration_ms=5000 />
            </Show>
        </div>
    }
}
