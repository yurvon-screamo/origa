use super::filter::Filter;
use super::filter_btn::FilterBtn;
use super::kanji_card_item::{CardStatus, KanjiCardItem};
use crate::ui_components::{Input, Text, TextSize, TypographyVariant};
use leptos::either::Either;
use leptos::prelude::*;
use origa::domain::{Card, User};

#[derive(Clone, Copy, PartialEq, Default)]
struct CardCounts {
    total: usize,
    new: usize,
    hard: usize,
    in_progress: usize,
    learned: usize,
}

#[component]
pub fn KanjiContent() -> impl IntoView {
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
                    .filter(|(_, card)| matches!(card.card(), Card::Kanji(_)))
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
                    let kanji_text = card.card().question().text().to_lowercase();
                    let description = card.card().answer().text().to_lowercase();
                    kanji_text.contains(&query) || description.contains(&query)
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

            <div class="space-y-2">
                {move || {
                    let cards = filtered_cards.get();
                    if cards.is_empty() {
                        Either::Left(view! {
                            <Text size=TextSize::Default variant=TypographyVariant::Muted>
                                "Кандзи не найдено"
                            </Text>
                        })
                    } else {
                        Either::Right(view! {
                            <For
                                each=move || filtered_cards.get()
                                key=|card| *card.card_id()
                                children=move |card| {
                                    view! { <KanjiCardItem study_card=card /> }
                                }
                            />
                        })
                    }
                }}
            </div>
        </div>
    }
}
