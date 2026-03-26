use super::super::shared::{create_delete_callback, CardCounts, CardStatus, Filter, FilterBtn};
use super::radical_item::RadicalItem;
use crate::repository::HybridUserRepository;
use crate::ui_components::{
    Input, LoadingOverlay, Text, TextSize, ToastContainer, ToastData, TypographyVariant,
};
use leptos::either::Either;
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::domain::{Card, NativeLanguage, StudyCard, User};
use origa::traits::UserRepository;
use origa::use_cases::ToggleFavoriteUseCase;
use ulid::Ulid;

fn load_user_data(
    repository: HybridUserRepository,
    current_user: RwSignal<Option<User>>,
    all_cards: RwSignal<Vec<StudyCard>>,
    is_loading: RwSignal<bool>,
) {
    spawn_local(async move {
        match repository.get_current_user().await {
            Ok(Some(user)) => {
                let cards = user
                    .knowledge_set()
                    .study_cards()
                    .iter()
                    .filter(|(_, card)| matches!(card.card(), Card::Radical(_)))
                    .map(|(_, card)| card.clone())
                    .collect();
                all_cards.set(cards);
                current_user.set(Some(user));
                is_loading.set(false);
            },
            Ok(None) => {
                tracing::warn!("RadicalsContent: user not found");
            },
            Err(e) => {
                tracing::error!("RadicalsContent: get_current_user error: {:?}", e);
            },
        }
    });
}

#[component]
pub fn RadicalsContent(refresh_trigger: RwSignal<u32>) -> impl IntoView {
    let repository =
        use_context::<HybridUserRepository>().expect("repository context not provided");

    let current_user: RwSignal<Option<User>> = RwSignal::new(None);
    let is_loading = RwSignal::new(true);
    let all_cards: RwSignal<Vec<StudyCard>> = RwSignal::new(Vec::new());

    let repo_for_init = repository.clone();
    Effect::new(move |_| {
        load_user_data(repo_for_init.clone(), current_user, all_cards, is_loading);
    });

    let repo_for_refresh = repository.clone();
    Effect::new(move |_| {
        let _ = refresh_trigger.get();
        load_user_data(
            repo_for_refresh.clone(),
            current_user,
            all_cards,
            is_loading,
        );
    });

    let native_lang = Memo::new(move |_| {
        current_user
            .get()
            .map(|u| *u.native_language())
            .unwrap_or(NativeLanguage::Russian)
    });

    let search = RwSignal::new(String::new());
    let filter = RwSignal::new(Filter::All);
    let toasts: RwSignal<Vec<ToastData>> = RwSignal::new(Vec::new());

    let on_toggle_favorite = {
        let repo = repository.clone();

        Callback::new(move |card_id: Ulid| {
            let repository = repo.clone();
            let user_signal = current_user;

            spawn_local(async move {
                let use_case = ToggleFavoriteUseCase::new(&repository);
                if use_case.execute(card_id).await.is_ok() {
                    user_signal.update(|u| {
                        if let Some(user) = u {
                            let _ = user.toggle_favorite(card_id);
                        }
                    });
                }
            });
        })
    };

    let (is_deleting, on_delete) = create_delete_callback(repository.clone(), toasts);

    let filtered_cards = Memo::new(move |_| {
        let query = search.get().to_lowercase();
        let current_filter = filter.get();
        let lang = native_lang.get();

        all_cards
            .get()
            .into_iter()
            .filter(|card| {
                let matches_search = query.is_empty() || {
                    let radical_text = card
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
                    radical_text.contains(&query) || meaning.contains(&query)
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
                <LoadingOverlay message=Signal::derive(|| "Загрузка...".to_string()) />
            </Show>
            <Show when=move || !is_loading.get()>
                <Input
                    value=search
                    placeholder=Signal::derive(|| "Поиск...".to_string())
                    test_id="radicals-search-input"
                />

                <div class="flex flex-wrap gap-2">
                    <FilterBtn filter=Filter::All count=move || counts.get().total active=filter />
                    <FilterBtn filter=Filter::New count=move || counts.get().new active=filter />
                    <FilterBtn filter=Filter::Hard count=move || counts.get().hard active=filter />
                    <FilterBtn filter=Filter::InProgress count=move || counts.get().in_progress active=filter />
                    <FilterBtn filter=Filter::Learned count=move || counts.get().learned active=filter />
                </div>

                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 2xl:grid-cols-5 gap-4" data-testid="radicals-grid">
                    {move || {
                        let cards = filtered_cards.get();
                        if cards.is_empty() {
                            Either::Left(view! {
                                <div class="col-span-full" data-testid="radicals-empty-state">
                                    <Text size=TextSize::Default variant=TypographyVariant::Muted>
                                        "Радикалы не найдены"
                                    </Text>
                                </div>
                            })
                        } else {
                            Either::Right(view! {
                                <For
                                    each=move || filtered_cards.get()
                                    key=|card| *card.card_id()
                                    children=move |card| {
                                        view! {
                                            <RadicalItem
                                                study_card=card
                                                native_language=native_lang.get()
                                                on_toggle_favorite=on_toggle_favorite
                                                on_delete=on_delete
                                                _is_deleting=Signal::derive(move || is_deleting.get())
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
