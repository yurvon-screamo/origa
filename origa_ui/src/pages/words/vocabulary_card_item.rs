use std::collections::HashSet;

use super::super::shared::{CardStatus, DeleteRequest};
use crate::ui_components::{
    Card, CardHistoryModal, CollapsibleDescription, DeleteButton, DeleteConfirmModal,
    FavoriteButton, FuriganaText, Heading, HeadingLevel, HistoryButton, MarkdownText, Tag, Text,
    TextSize, TypographyVariant,
};
use leptos::prelude::*;
use origa::domain::{Card as DomainCard, NativeLanguage, StudyCard};
use ulid::Ulid;

#[component]
pub fn VocabularyCardItem(
    study_card: StudyCard,
    native_language: NativeLanguage,
    known_kanji: HashSet<String>,
    on_toggle_favorite: Callback<Ulid>,
    on_delete: Callback<DeleteRequest>,
    is_deleting: Signal<bool>,
) -> impl IntoView {
    let card = study_card.card();
    let card_id = *study_card.card_id();
    let is_favorite = study_card.is_favorite();
    let memory = study_card.memory();
    let memory_clone = memory.clone();

    let is_history_open = RwSignal::new(false);
    let is_delete_modal_open = RwSignal::new(false);
    let confirm_delete = Callback::new(move |_| {
        on_delete.run(DeleteRequest {
            card_id,
            on_success: Callback::new(move |_| is_delete_modal_open.set(false)),
        })
    });

    let (word, meaning) = match card {
        DomainCard::Vocabulary(vocab) => (
            vocab.word().text().to_string(),
            vocab
                .answer(&native_language)
                .ok()
                .map(|a| a.text().to_string())
                .unwrap_or_default(),
        ),
        _ => ("?".to_string(), "?".to_string()),
    };

    let status = CardStatus::from_study_card(&study_card);

    let known_kanji_for_furigana = known_kanji;
    let known_kanji_for_markdown = known_kanji_for_furigana.clone();

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

    view! {
        <Card class="p-4" test_id="words-card-item">
            <Heading level=HeadingLevel::H4 class="mb-2">
                <FuriganaText text=word.clone() known_kanji=known_kanji_for_furigana/>
            </Heading>
            <CollapsibleDescription>
                <MarkdownText content=Signal::derive(move || meaning.clone()) known_kanji=known_kanji_for_markdown/>
            </CollapsibleDescription>
            <Text
                size=TextSize::Small
                variant=TypographyVariant::Muted
                class="mt-2"
            >
                {format!("Повтор: {} | Слож: {} | Стаб: {}", next_review, difficulty, stability)}
            </Text>
            <div class="border-t border-[var(--border-dark)] pt-2 mt-2 flex justify-between items-center">
                <Tag variant=Signal::derive(move || status.tag_variant()) test_id="words-card-tag">
                    {status.label()}
                </Tag>
                <div class="flex items-center gap-2">
                    <FavoriteButton
                        is_favorite=Signal::derive(move || is_favorite)
                        on_click=Callback::new(move |_| on_toggle_favorite.run(card_id))
                    />
                    <HistoryButton on_click=Callback::new(move |_| is_history_open.set(true)) />
                    <DeleteButton on_click=Callback::new(move |_| is_delete_modal_open.set(true)) />
                </div>
            </div>
            <CardHistoryModal
                is_open=Signal::derive(move || is_history_open.get())
                memory=memory_clone.clone()
                on_close=Callback::new(move |_| is_history_open.set(false))
            />
            <DeleteConfirmModal
                test_id="words-delete-modal"
                is_open=is_delete_modal_open
                is_deleting=is_deleting
                on_confirm=confirm_delete
                on_close=Callback::new(move |_| is_delete_modal_open.set(false))
            />
        </Card>
    }
}
