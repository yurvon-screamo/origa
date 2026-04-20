use std::collections::HashSet;

use super::super::shared::{CardStatus, DeleteRequest, MarkAsKnownButton};
use crate::i18n::use_i18n;
use crate::ui_components::{
    AudioPlayer, Card, CardHistoryModal, CollapsibleDescription, DeleteButton, DeleteConfirmModal,
    FavoriteButton, FuriganaText, Heading, HeadingLevel, HistoryButton, MarkdownText, Tag, Text,
    TextSize, TypographyVariant,
};
use leptos::prelude::*;
use origa::domain::{Card as DomainCard, NativeLanguage, StudyCard};
use ulid::Ulid;

#[component]
pub fn PhraseCardItem(
    study_card: StudyCard,
    #[prop(into)] native_language: Signal<NativeLanguage>,
    known_kanji: HashSet<String>,
    on_toggle_favorite: Callback<Ulid>,
    on_mark_as_known: Callback<()>,
    on_delete: Callback<DeleteRequest>,
    is_deleting: Signal<bool>,
) -> impl IntoView {
    let i18n = use_i18n();
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

    let (phrase_text, audio_src) = match study_card.card() {
        DomainCard::Phrase(phrase_card) => {
            let text = phrase_card.question().unwrap_or_default();
            let src = crate::core::config::cdn_url(&format!(
                "/phrases/audio/{}.opus",
                phrase_card.phrase_id()
            ));
            (text, src)
        },
        _ => (String::new(), String::new()),
    };

    let study_card_for_meaning = study_card.clone();
    let meaning = Memo::new(move |_| {
        let lang = native_language.get();
        match study_card_for_meaning.card() {
            DomainCard::Phrase(phrase_card) => phrase_card.answer(&lang).unwrap_or_default(),
            _ => String::new(),
        }
    });

    let status = CardStatus::from_study_card(&study_card);
    let has_audio = !audio_src.is_empty();

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

    let known_kanji_for_furigana = known_kanji;
    let known_kanji_for_markdown = known_kanji_for_furigana.clone();

    view! {
        <Card class="p-4" test_id="phrases-card-item">
            <div class="flex items-start justify-between gap-2 mb-2">
                <Heading level=HeadingLevel::H4 class="flex-1">
                    <FuriganaText text=phrase_text.clone() known_kanji=known_kanji_for_furigana/>
                </Heading>
                <Show when=move || has_audio>
                    <AudioPlayer
                        src=audio_src.clone()
                        autoplay=false
                        test_id=Signal::derive(|| "phrases-card-audio".to_string())
                    />
                </Show>
            </div>
            <CollapsibleDescription>
                <MarkdownText content=Signal::derive(move || meaning.get()) known_kanji=known_kanji_for_markdown/>
            </CollapsibleDescription>
            <Text
                size=TextSize::Small
                variant=TypographyVariant::Muted
                class="mt-2"
            >
                {move || {
                    i18n.get_keys().shared().card_info().inner().to_string()
                        .replacen("{}", &next_review, 1)
                        .replacen("{}", &difficulty, 1)
                        .replacen("{}", &stability, 1)
                }}
            </Text>
            <div class="border-t border-[var(--border-dark)] pt-2 mt-2 flex justify-between items-center">
                <Tag variant=Signal::derive(move || status.tag_variant()) test_id="phrases-card-tag">
                    {move || status.label(&i18n)}
                </Tag>
                <div class="flex items-center gap-2">
                    <FavoriteButton
                        is_favorite=Signal::derive(move || is_favorite)
                        on_click=Callback::new(move |_| on_toggle_favorite.run(card_id))
                    />
                    <Show when=move || status != CardStatus::Learned>
                        <MarkAsKnownButton
                            on_click=Callback::new(move |_| on_mark_as_known.run(()))
                            test_id=Signal::derive(|| "phrases-card-item-mark-known-btn".to_string())
                        />
                    </Show>
                    <HistoryButton on_click=Callback::new(move |_| is_history_open.set(true)) />
                    <DeleteButton test_id="phrases-card-item-delete-btn" on_click=Callback::new(move |_| is_delete_modal_open.set(true)) />
                </div>
            </div>
            <CardHistoryModal
                is_open=Signal::derive(move || is_history_open.get())
                memory=memory_clone.clone()
                on_close=Callback::new(move |_| is_history_open.set(false))
            />
            <DeleteConfirmModal
                test_id="phrases-delete-modal"
                is_open=is_delete_modal_open
                is_deleting=is_deleting
                on_confirm=confirm_delete
                on_close=Callback::new(move |_| is_delete_modal_open.set(false))
            />
        </Card>
    }
}
