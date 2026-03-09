use super::super::shared::{CardCounts, CardStatus, Filter, FilterBtn};
use super::vocabulary_card_item::VocabularyCardItem;
use crate::repository::HybridUserRepository;
use crate::ui_components::{Input, Text, TextSize, TypographyVariant};
use leptos::either::Either;
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::domain::{Card, NativeLanguage, User};
use origa::use_cases::ToggleFavoriteUseCase;
use ulid::Ulid;

#[component]
pub fn WordsContent() -> impl IntoView {
    let current_user =
        use_context::<RwSignal<Option<User>>>().expect("current_user context not provided");

    let native_lang = Memo::new(move |_| {
        current_user
            .get()
            .map(|u| *u.native_language())
            .unwrap_or(NativeLanguage::Russian)
    });

    let search = RwSignal::new(String::new());
    let filter = RwSignal::new(Filter::All);

    let repository =
        use_context::<HybridUserRepository>().expect("repository context not provided");

    let on_toggle_favorite = {
        let repository = repository.clone();

        Callback::new(move |card_id: Ulid| {
            let user = current_user.get();
            let repo = repository.clone();
            let current_user_clone = current_user;

            if let Some(user) = user {
                let user_id = user.id();
                spawn_local(async move {
                    let use_case = ToggleFavoriteUseCase::new(&repo);
                    if use_case.execute(user_id, card_id).await.is_ok() {
                        current_user_clone.update(|u| {
                            if let Some(user) = u {
                                let _ = user.toggle_favorite(card_id);
                            }
                        });
                    }
                });
            }
        })
    };

    let all_cards = Memo::new(move |_| {
        current_user
            .get()
            .map(|user| {
                user.knowledge_set()
                    .study_cards()
                    .iter()
                    .filter(|(_, card)| matches!(card.card(), Card::Vocabulary(_)))
                    .map(|(_, card)| card.clone())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    });

    let filtered_cards = Memo::new(move |_| {
        let query = search.get().to_lowercase();
        let current_filter = filter.get();
        let lang = native_lang.get();

        all_cards
            .get()
            .into_iter()
            .filter(|card| {
                let matches_search = query.is_empty() || {
                    let word = card.card().question(&lang).text().to_lowercase();
                    let meaning = card.card().answer(&lang).text().to_lowercase();
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

            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                {move || {
                    let cards = filtered_cards.get();
                    if cards.is_empty() {
                        Either::Left(view! {
                            <div class="col-span-full">
                                <Text size=TextSize::Default variant=TypographyVariant::Muted>
                                    "Слов не найдено"
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
                                        <VocabularyCardItem
                                            study_card=card
                                            on_toggle_favorite=on_toggle_favorite
                                        />
                                    }
                                }
                            />
                        })
                    }
                }}
            </div>
        </div>
    }
}
