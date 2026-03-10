use super::super::shared::CardStatus;
use crate::ui_components::{
    Card, CardHistoryModal, CollapsibleDescription, DeleteButton, DeleteConfirmModal,
    FavoriteButton, FuriganaText, Heading, HeadingLevel, HistoryButton, MarkdownText, Tag, Text,
    TextSize, TypographyVariant,
};
use leptos::prelude::*;
use origa::domain::{Card as DomainCard, NativeLanguage, StudyCard, User};
use ulid::Ulid;

#[component]
pub fn VocabularyCardItem(
    study_card: StudyCard,
    on_toggle_favorite: Callback<Ulid>,
    on_delete: Callback<Ulid>,
    is_deleting: Signal<bool>,
) -> impl IntoView {
    let current_user = use_context::<RwSignal<Option<User>>>().expect("current_user context");

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

    let card = study_card.card();
    let card_id = *study_card.card_id();
    let is_favorite = study_card.is_favorite();
    let memory = study_card.memory();
    let memory_clone = memory.clone();

    let is_history_open = RwSignal::new(false);
    let is_delete_modal_open = RwSignal::new(false);
    let confirm_delete = Callback::new(move |_| on_delete.run(card_id));

    let lang = native_lang.get();
    let (word, meaning) = match card {
        DomainCard::Vocabulary(vocab) => (
            vocab.word().text().to_string(),
            vocab
                .answer(&lang)
                .ok()
                .map(|a| a.text().to_string())
                .unwrap_or_default(),
        ),
        _ => ("?".to_string(), "?".to_string()),
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

    view! {
        <Card class=Signal::derive(|| "p-4".to_string())>
            <div class="flex justify-between items-start">
                <div class="min-w-0 flex-1">
                    <div class="flex items-center gap-3 mb-2">
                        <Heading level=HeadingLevel::H4>
                            <FuriganaText text=word.clone() known_kanji=known_kanji.get()/>
                        </Heading>
                        <Tag variant=Signal::derive(move || status.tag_variant())>
                            {status.label()}
                        </Tag>
                        <FavoriteButton
                            is_favorite=Signal::derive(move || is_favorite)
                            on_click=Callback::new(move |_| on_toggle_favorite.run(card_id))
                        />
                        <HistoryButton on_click=Callback::new(move |_| is_history_open.set(true)) />
                        <DeleteButton on_click=Callback::new(move |_| is_delete_modal_open.set(true)) />
                    </div>
                    <CollapsibleDescription>
                        <MarkdownText content=Signal::derive(move || meaning.clone()) known_kanji=known_kanji.get()/>
                    </CollapsibleDescription>

                    <Text
                        size=TextSize::Small
                        variant=TypographyVariant::Muted
                        class=Signal::derive(|| "mt-2".to_string())
                    >
                        {format!("Повтор: {} | Слож: {} | Стаб: {}", next_review, difficulty, stability)}
                    </Text>
                </div>
            </div>
            <CardHistoryModal
                is_open=Signal::derive(move || is_history_open.get())
                memory=memory_clone.clone()
                on_close=Callback::new(move |_| is_history_open.set(false))
            />
            <DeleteConfirmModal
                is_open=is_delete_modal_open
                is_deleting=is_deleting
                on_confirm=confirm_delete
                on_close=Callback::new(move |_| is_delete_modal_open.set(false))
            />
        </Card>
    }
}
