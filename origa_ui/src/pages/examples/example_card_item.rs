use super::super::shared::{CardItemActions, CardStatus, DeleteRequest};
use crate::i18n::use_i18n;
use crate::loaders::example_loader::example_audio_path;
use crate::ui_components::{
    AudioPlayer, CardActionBar, DeleteConfirmModal, FsrsMetrics, Skeleton, Tag, TagVariant, Text,
    TextSize, TranslatorText, TypographyVariant,
};
use leptos::prelude::*;
use origa::domain::{Card as DomainCard, NativeLanguage, StudyCard};

#[component]
pub fn ExampleCardItem(
    study_card: StudyCard,
    #[prop(into)] native_language: Signal<NativeLanguage>,
    actions: CardItemActions,
    details_trigger: RwSignal<u32>,
) -> impl IntoView {
    let i18n = use_i18n();
    let card_id = *study_card.card_id();
    let is_favorite = study_card.is_favorite();
    let memory = study_card.memory();

    let is_delete_modal_open = RwSignal::new(false);

    let confirm_delete = Callback::new(move |_| {
        actions.on_delete.run(DeleteRequest {
            card_id,
            on_success: Callback::new(move |_| is_delete_modal_open.set(false)),
        })
    });

    let (sentence_id, owner_word) = match study_card.card() {
        DomainCard::Example(ec) => (ec.sentence_id(), ec.word().to_string()),
        _ => (0, String::new()),
    };

    let study_card_for_text = study_card.clone();
    let sentence_text: Memo<String> = Memo::new(move |_| {
        let _ = details_trigger.get();
        match study_card_for_text.card() {
            DomainCard::Example(ec) => {
                get_cached_sentence_text(ec.sentence_id()).unwrap_or_default()
            },
            _ => String::new(),
        }
    });

    // Bug A fix (phrases etalon): pass the CDN path (not the resolved URL)
    // to AudioPlayer so it can prefetch into a `blob:` URL.
    let audio_path = example_audio_path(sentence_id);

    let study_card_for_meaning = study_card.clone();
    let meaning: Memo<String> = Memo::new(move |_| {
        let _ = details_trigger.get();
        let lang = native_language.get();
        match study_card_for_meaning.card() {
            DomainCard::Example(ec) => {
                get_cached_sentence_translation(ec.sentence_id(), &lang).unwrap_or_default()
            },
            _ => String::new(),
        }
    });

    let status = CardStatus::from_study_card(&study_card);

    view! {
        <div class="phrase-card" data-testid="examples-card-item">
            <div class="phrase-card-body">
                <div class="phrase-card-header">
                    <div class="phrase-card-phrase" data-testid="examples-card-sentence">
                        {move || {
                            let text = sentence_text.get();
                            if text.is_empty() {
                                view! {
                                    <Skeleton
                                        width="60%".to_string()
                                        height="1.5em".to_string()
                                        test_id=Signal::derive(|| "examples-card-text-skeleton".to_string())
                                    />
                                }
                                .into_any()
                            } else {
                                view! {
                                    <TranslatorText
                                        text=text
                                        native_language=native_language
                                        test_id=Signal::derive(|| "examples-card-text".to_string())
                                    />
                                }
                                .into_any()
                            }
                        }}
                    </div>
                    <div class="phrase-card-meta">
                        <div class="phrase-card-badge">
                            <Tag variant=Signal::derive(move || status.tag_variant())>
                                {move || status.label(&i18n)}
                            </Tag>
                        </div>
                        <div class="phrase-card-audio">
                            <AudioPlayer
                                path=audio_path.clone()
                                autoplay=false
                                test_id=Signal::derive(|| "examples-card-audio".to_string())
                            />
                        </div>
                    </div>
                </div>
                <div class="phrase-card-content">
                    {move || {
                        let translation = meaning.get();
                        if translation.is_empty() {
                            ().into_any()
                        } else {
                            view! {
                                <Text
                                    size=TextSize::Default
                                    variant=TypographyVariant::Primary
                                    test_id=Signal::derive(|| "examples-card-translation".to_string())
                                >
                                    {translation}
                                </Text>
                            }
                            .into_any()
                        }
                    }}
                </div>
            </div>
            <div class="phrase-card-divider"></div>
            <div class="phrase-card-footer">
                <FsrsMetrics
                    difficulty=memory.difficulty().map(|d| d.value())
                    stability=memory.stability().map(|s| s.value())
                    test_id=Signal::derive(|| "examples-card-fsrs".to_string())
                />
                <div class="phrase-card-actions">
                    <Tag variant=TagVariant::Default test_id=Signal::derive(|| "examples-card-word".to_string())>
                        {owner_word.clone()}
                    </Tag>
                    <CardActionBar
                        tag_variant=TagVariant::default()
                        tag_label=Signal::derive(|| "".to_string())
                        is_favorite=Signal::derive(move || is_favorite)
                        on_toggle_favorite=Callback::new(move |_| {
                            actions.on_toggle_favorite.run(card_id)
                        })
                        show_mark_as_known=Signal::derive(move || status != CardStatus::Learned)
                        on_mark_as_known=Callback::new(move |_| actions.on_mark_as_known.run(card_id))
                        on_delete=Callback::new(move |_| is_delete_modal_open.set(true))
                        test_id=Signal::derive(|| "examples-card-item".to_string())
                        show_tag=Signal::derive(|| false)
                    />
                </div>
            </div>
        </div>
        <DeleteConfirmModal
            test_id="examples-delete-modal"
            is_open=is_delete_modal_open
            is_deleting=actions.is_deleting
            on_confirm=confirm_delete
            on_close=Callback::new(move |_| is_delete_modal_open.set(false))
        />
    }
}

fn get_cached_sentence_text(sentence_id: u32) -> Option<String> {
    origa::dictionary::example::get_cached_example_detail(sentence_id).map(|d| d.text)
}

fn get_cached_sentence_translation(sentence_id: u32, lang: &NativeLanguage) -> Option<String> {
    origa::dictionary::example::get_cached_example_detail(sentence_id)
        .and_then(|d| d.translation(lang).map(str::to_string))
}
