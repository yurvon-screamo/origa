use super::super::shared::{CardCounts, CardStatus, Filter, FilterBtn, create_delete_callback};
use super::grammar_card_item::GrammarCardItem;
use crate::repository::HybridUserRepository;
use crate::ui_components::{
    Input, LoadingOverlay, Text, TextSize, ToastContainer, ToastData, TypographyVariant,
};
use leptos::either::Either;
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::domain::{NativeLanguage, StudyCard, User};
use origa::traits::UserRepository;
use origa::use_cases::ToggleFavoriteUseCase;
use ulid::Ulid;

#[component]
pub fn GrammarContent(refresh_trigger: RwSignal<u32>) -> impl IntoView {
    let repository =
        use_context::<HybridUserRepository>().expect("repository context not provided");

    let current_user: RwSignal<Option<User>> = RwSignal::new(None);
    let is_loading = RwSignal::new(true);
    let all_cards: RwSignal<Vec<StudyCard>> = RwSignal::new(Vec::new());
    let repo_for_init = repository.clone();

    Effect::new(move |_| {
        let _ = refresh_trigger.get();
        let repo = repo_for_init.clone();
        spawn_local(async move {
            match repo.get_current_user().await {
                Ok(Some(user)) => {
                    let cards = user
                        .knowledge_set()
                        .study_cards()
                        .iter()
                        .filter(|(_, card)| matches!(card.card(), origa::domain::Card::Grammar(_)))
                        .map(|(_, card)| card.clone())
                        .collect();
                    all_cards.set(cards);
                    current_user.set(Some(user));
                    is_loading.set(false);
                }
                Ok(None) => {
                    tracing::warn!("GrammarContent: user not found");
                }
                Err(e) => {
                    tracing::error!("GrammarContent: get_current_user error: {:?}", e);
                }
            }
        });
    });

    let native_lang = Memo::new(move |_| {
        current_user
            .get()
            .map(|u| *u.native_language())
            .unwrap_or(NativeLanguage::Russian)
    });

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
                    let card = card.card();
                    let question = card.question(&lang);
                    let answer = card.answer(&lang);

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
                />

                <div class="flex flex-wrap gap-2">
                    <FilterBtn filter=Filter::All count=move || counts.get().total active=filter />
                    <FilterBtn filter=Filter::New count=move || counts.get().new active=filter />
                    <FilterBtn filter=Filter::Hard count=move || counts.get().hard active=filter />
                    <FilterBtn filter=Filter::InProgress count=move || counts.get().in_progress active=filter />
                    <FilterBtn filter=Filter::Learned count=move || counts.get().learned active=filter />
                </div>

                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 2xl:grid-cols-5 gap-4">
                    {move || {
                        let cards = filtered_cards.get();
                        if cards.is_empty() {
                            Either::Left(view! {
                                <div class="col-span-full">
                                    <Text size=TextSize::Default variant=TypographyVariant::Muted>
                                        "Грамматических конструкций не найдено"
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
                                            <GrammarCardItem
                                                study_card=card
                                                native_language=native_lang.get()
                                                known_kanji=known_kanji.get()
                                                on_toggle_favorite=on_toggle_favorite
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
