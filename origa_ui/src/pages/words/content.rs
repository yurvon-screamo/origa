use super::filter::Filter;
use super::filter_btn::FilterBtn;
use super::vocabulary_card_item::{CardStatus, VocabularyCardItem};
use crate::ui_components::{Input, Text, TextSize, TypographyVariant};
use leptos::prelude::*;
use origa::domain::{Card, User};

#[component]
pub fn WordsContent() -> impl IntoView {
    let current_user =
        use_context::<RwSignal<Option<User>>>().expect("current_user context not provided");

    let search = RwSignal::new(String::new());
    let filter = RwSignal::new(Filter::All);

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

        all_cards
            .get()
            .into_iter()
            .filter(|card| {
                let matches_search = query.is_empty() || {
                    let word = card.card().question().text().to_lowercase();
                    let meaning = card.card().answer().text().to_lowercase();
                    word.contains(&query) || meaning.contains(&query)
                };
                let matches_filter = current_filter.matches(CardStatus::from_study_card(card));
                matches_search && matches_filter
            })
            .collect::<Vec<_>>()
    });

    let counts = Memo::new(move |_| {
        let cards = all_cards.get();
        let new = cards
            .iter()
            .filter(|c| CardStatus::from_study_card(c) == CardStatus::New)
            .count();
        let hard = cards
            .iter()
            .filter(|c| CardStatus::from_study_card(c) == CardStatus::Hard)
            .count();
        let progress = cards
            .iter()
            .filter(|c| CardStatus::from_study_card(c) == CardStatus::InProgress)
            .count();
        let learned = cards
            .iter()
            .filter(|c| CardStatus::from_study_card(c) == CardStatus::Learned)
            .count();
        (cards.len(), new, hard, progress, learned)
    });

    view! {
        <div class="space-y-4">
            <Input
                value=search
                placeholder=Signal::derive(|| "Поиск...".to_string())
            />

            <div class="flex flex-wrap gap-2">
                <FilterBtn filter=Filter::All count=move || counts.get().0 active=filter />
                <FilterBtn filter=Filter::New count=move || counts.get().1 active=filter />
                <FilterBtn filter=Filter::Hard count=move || counts.get().2 active=filter />
                <FilterBtn filter=Filter::InProgress count=move || counts.get().3 active=filter />
                <FilterBtn filter=Filter::Learned count=move || counts.get().4 active=filter />
            </div>

            <div class="space-y-2">
                {move || {
                    let cards = filtered_cards.get();
                    if cards.is_empty() {
                        view! {
                            <Text size=TextSize::Default variant=TypographyVariant::Muted>
                                "Слов не найдено"
                            </Text>
                        }.into_any()
                    } else {
                        cards.into_iter()
                            .map(|card| view! { <VocabularyCardItem study_card=card /> }.into_any())
                            .collect::<Vec<_>>()
                            .into_any()
                    }
                }}
            </div>
        </div>
    }
}
