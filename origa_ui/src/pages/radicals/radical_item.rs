use super::super::shared::{CardStatus, DeleteRequest};
use crate::ui_components::{
    Card, CardHistoryModal, DeleteButton, FavoriteButton, HistoryButton, MarkdownText, Tag, Text,
    TextSize, TypographyVariant,
};
use leptos::prelude::*;
use origa::domain::{Card as DomainCard, NativeLanguage, StudyCard};
use std::collections::HashSet;
use ulid::Ulid;

#[component]
pub fn RadicalItem(
    study_card: StudyCard,
    native_language: NativeLanguage,
    on_toggle_favorite: Callback<Ulid>,
    on_delete: Callback<DeleteRequest>,
    #[allow(unused_variables)] is_deleting: Signal<bool>,
) -> impl IntoView {
    let card = study_card.card();
    let card_id = *study_card.card_id();
    let is_favorite = study_card.is_favorite();
    let memory = study_card.memory();
    let memory_clone = memory.clone();

    let is_history_open = RwSignal::new(false);

    let question = card
        .question(&native_language)
        .ok()
        .map(|q| q.text().to_string())
        .unwrap_or_default();

    let answer = card
        .answer(&native_language)
        .ok()
        .map(|a| a.text().to_string())
        .unwrap_or_default();

    let description = match card {
        DomainCard::Radical(radical_card) => radical_card
            .radical_info()
            .map(|i| i.description().to_string())
            .unwrap_or_default(),
        _ => String::new(),
    };

    let status = CardStatus::from_study_card(&study_card);

    let difficulty = memory
        .difficulty()
        .map(|d| format!("{:.1}", d.value()))
        .unwrap_or("-".to_string());
    let stability = memory
        .stability()
        .map(|s| format!("{:.1}", s.value()))
        .unwrap_or("-".to_string());
    let next_review = memory
        .next_review_date()
        .map(|d| d.format("%d.%m.%Y").to_string())
        .unwrap_or("-".to_string());

    let known_kanji: HashSet<String> = HashSet::new();

    view! {
        <Card class="p-4">
            <div class="flex items-start gap-3 mb-2">
                <span class="text-3xl font-serif">{question}</span>
                <div class="min-w-0 flex-1">
                    <MarkdownText content=Signal::derive(move || description.clone()) known_kanji=known_kanji.clone()/>
                </div>
            </div>
            <MarkdownText content=Signal::derive(move || answer.clone()) known_kanji=known_kanji.clone()/>
            <Text size=TextSize::Small variant=TypographyVariant::Muted class="mt-2">
                {format!("Повтор: {} | Слож: {} | Стаб: {}", next_review, difficulty, stability)}
            </Text>
            <CardHistoryModal
                is_open=Signal::derive(move || is_history_open.get())
                memory=memory_clone.clone()
                on_close=Callback::new(move |_| is_history_open.set(false))
            />
            <div class="border-t border-[var(--border-dark)] pt-2 mt-2 flex justify-between items-center">
                <Tag variant=Signal::derive(move || status.tag_variant())>
                    {status.label()}
                </Tag>
                <div class="flex gap-1">
                    <FavoriteButton
                        is_favorite=Signal::derive(move || is_favorite)
                        on_click=Callback::new(move |_| on_toggle_favorite.run(card_id))
                    />
                    <HistoryButton on_click=Callback::new(move |_| is_history_open.set(true)) />
                    <DeleteButton
                        on_click=Callback::new(move |_| {
                            on_delete.run(DeleteRequest {
                                card_id,
                                on_success: Callback::new(move |_| ()),
                            });
                        })
                    />
                </div>
            </div>
        </Card>
    }
}
