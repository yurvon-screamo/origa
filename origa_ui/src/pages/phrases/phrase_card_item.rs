use std::collections::HashSet;

use super::super::shared::{CardStatus, DeleteRequest};
use crate::i18n::use_i18n;
use crate::ui_components::{
    AudioPlayer, CardActionBar, CardHistoryModal, CollapsibleDescription, DeleteConfirmModal,
    FsrsMetrics, MarkdownText, Skeleton, Tag, TagVariant, TranslatorText,
};
use leptos::prelude::*;
use origa::domain::{Card as DomainCard, NativeLanguage, StudyCard};
use ulid::Ulid;

#[component]
pub fn PhraseCardItem(
    study_card: StudyCard,
    #[prop(into)] native_language: Signal<NativeLanguage>,
    known_kanji: HashSet<char>,
    on_toggle_favorite: Callback<Ulid>,
    on_mark_as_known: Callback<()>,
    on_delete: Callback<DeleteRequest>,
    is_deleting: Signal<bool>,
    phrase_data_trigger: RwSignal<u32>,
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

    let study_card_for_text = study_card.clone();
    let phrase_text: Memo<String> = Memo::new(move |_| {
        let _ = phrase_data_trigger.get();
        match study_card_for_text.card() {
            DomainCard::Phrase(phrase_card) => phrase_card.question().unwrap_or_default(),
            _ => String::new(),
        }
    });

    let audio_src = match study_card.card() {
        DomainCard::Phrase(phrase_card) => crate::repository::cdn_provider::resolve_audio_url(
            &format!("phrases/audio/{}.opus", phrase_card.phrase_id()),
        ),
        _ => String::new(),
    };

    let study_card_for_meaning = study_card.clone();
    let meaning = Memo::new(move |_| {
        let _ = phrase_data_trigger.get();
        let lang = native_language.get();
        match study_card_for_meaning.card() {
            DomainCard::Phrase(phrase_card) => phrase_card.answer(&lang).unwrap_or_default(),
            _ => String::new(),
        }
    });

    let status = CardStatus::from_study_card(&study_card);
    let has_audio = !audio_src.is_empty();
    let known_kanji_for_markdown = known_kanji;

    view! {
        <div class="phrase-card anima-lift" data-testid="phrases-card-item">
            <div class="phrase-card-badge">
                <Tag variant=Signal::derive(move || status.tag_variant())>
                    {move || status.label(&i18n)}
                </Tag>
            </div>
            <div class="phrase-card-body">
                <div class="phrase-card-header">
                    <div class="phrase-card-phrase" data-testid="phrases-card-phrase">
                        {move || {
                            let text = phrase_text.get();
                            if text.is_empty() {
                                view! {
                                    <Skeleton
                                        width="60%".to_string()
                                        height="1.5em".to_string()
                                        test_id=Signal::derive(|| "phrases-card-text-skeleton".to_string())
                                    />
                                }
                                .into_any()
                            } else {
                                view! {
                                    <TranslatorText
                                        text=text
                                        native_language=native_language
                                        test_id=Signal::derive(|| "phrases-card-text".to_string())
                                    />
                                }
                                .into_any()
                            }
                        }}
                    </div>
                    <Show when=move || has_audio>
                        <div class="phrase-card-audio">
                            <AudioPlayer
                                src=audio_src.clone()
                                autoplay=false
                                test_id=Signal::derive(|| "phrases-card-audio".to_string())
                            />
                        </div>
                    </Show>
                </div>
                <div class="phrase-card-content">
                    <CollapsibleDescription>
                        <MarkdownText
                            content=Signal::derive(move || meaning.get())
                            known_kanji=known_kanji_for_markdown
                            test_id=Signal::derive(|| "phrases-card-meaning".to_string())
                        />
                    </CollapsibleDescription>
                </div>
            </div>
            <div class="phrase-card-divider"></div>
            <div class="phrase-card-footer">
                <FsrsMetrics
                    difficulty=memory.difficulty().map(|d| d.value())
                    stability=memory.stability().map(|s| s.value())
                    test_id=Signal::derive(|| "phrases-card-fsrs".to_string())
                />
                <div class="phrase-card-actions">
                    <CardActionBar
                        tag_variant=TagVariant::default()
                        tag_label=Signal::derive(|| "".to_string())
                        is_favorite=Signal::derive(move || is_favorite)
                        on_toggle_favorite=Callback::new(move |_| on_toggle_favorite.run(card_id))
                        show_mark_as_known=Signal::derive(move || status != CardStatus::Learned)
                        on_mark_as_known=Callback::new(move |_| on_mark_as_known.run(()))
                        on_history=Callback::new(move |_| is_history_open.set(true))
                        on_delete=Callback::new(move |_| is_delete_modal_open.set(true))
                        test_id=Signal::derive(|| "phrases-card-item".to_string())
                        show_tag=Signal::derive(|| false)
                    />
                </div>
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
    }
}
