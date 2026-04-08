use std::collections::HashSet;

use super::super::shared::{CardStatus, DeleteRequest, MarkAsKnownButton};
use super::DrawingDrawer;
use crate::i18n::{t, use_i18n};
use crate::ui_components::{
    Button, ButtonSize, ButtonVariant, Card, CardHistoryModal, DeleteButton, DeleteConfirmModal,
    FavoriteButton, HistoryButton, KanjiViewMode, KanjiWritingSection, MarkdownText, Tag, Text,
    TextSize, TypographyVariant,
};
use leptos::{ev::MouseEvent, prelude::*};
use origa::domain::{Card as DomainCard, NativeLanguage, StudyCard};
use ulid::Ulid;

#[component]
pub fn KanjiCardItem(
    study_card: StudyCard,
    native_language: NativeLanguage,
    known_kanji: HashSet<String>,
    on_toggle_favorite: Callback<Ulid>,
    on_mark_as_known: Callback<()>,
    on_delete: Callback<DeleteRequest>,
    is_deleting: Signal<bool>,
) -> impl IntoView {
    let i18n = use_i18n();
    let card = study_card.card();
    let card_id = *study_card.card_id();
    let is_favorite = study_card.is_favorite();
    let memory = study_card.memory();
    let memory_clone = memory.clone();

    let is_history_open = RwSignal::new(false);
    let is_delete_modal_open = RwSignal::new(false);
    let drawing_drawer_open = RwSignal::new(false);
    let confirm_delete = Callback::new(move |_| {
        on_delete.run(DeleteRequest {
            card_id,
            on_success: Callback::new(move |_| is_delete_modal_open.set(false)),
        })
    });

    let (kanji_char, description, radicals, example_words) = match card {
        DomainCard::Kanji(kanji_card) => {
            let radicals_str = kanji_card.radicals_chars().into_iter().collect::<String>();
            let examples_str = kanji_card
                .example_words(&native_language)
                .iter()
                .map(|w| format!("{} ({})", w.word(), w.meaning()))
                .collect::<Vec<_>>()
                .join(", ");
            (
                kanji_card.kanji().text().to_string(),
                kanji_card
                    .description()
                    .ok()
                    .map(|d| d.text().to_string())
                    .unwrap_or_default(),
                radicals_str,
                examples_str,
            )
        },
        _ => (
            "?".to_string(),
            "?".to_string(),
            "".to_string(),
            "".to_string(),
        ),
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

    let kanji_for_animation = StoredValue::new(kanji_char.clone());
    let is_expanded = RwSignal::new(false);
    let has_examples = !example_words.is_empty();

    view! {
        <Card class="p-4" test_id="kanji-card-item">
            <div class="flex items-start gap-3 mb-2">
                <span class="text-3xl font-serif">{kanji_char.clone()}</span>
                <div class="min-w-0 flex-1">
                    <MarkdownText content=Signal::derive(move || description.clone()) known_kanji=known_kanji.clone()/>
                </div>
            </div>
            {move || {
                let radicals = radicals.clone();
                if !radicals.is_empty() {
                    view! {
                        <Text size=TextSize::Small variant=TypographyVariant::Muted class="mb-1">
                            {i18n.get_keys().shared().radicals_label().inner().to_string().replacen("{}", &radicals, 1)}
                        </Text>
                    }.into_any()
                } else {
                    ().into_any()
                }
            }}
            {move || {
                if has_examples && is_expanded.get() {
                    let examples = example_words.clone();
                    view! {
                        <div class="mb-1">
                            <MarkdownText content=Signal::derive(move || format!("**{}** {}", i18n.get_keys().shared().examples_label().inner().to_string().replacen("{}", &examples, 1), "")) known_kanji=known_kanji.clone()/>
                        </div>
                    }.into_any()
                } else {
                    ().into_any()
                }
            }}
            <Text
                size=TextSize::Small
                variant=TypographyVariant::Muted
                class="mt-2"
            >
                {i18n.get_keys().shared().card_info().inner().to_string()
                    .replacen("{}", &next_review, 1)
                    .replacen("{}", &difficulty, 1)
                    .replacen("{}", &stability, 1)}
            </Text>
            <Show when=move || has_examples>
                <div class="mt-1 flex items-center gap-3">
                    <Button
                        variant=ButtonVariant::Ghost
                        size=ButtonSize::Small
                        on_click=Callback::new(move |_: MouseEvent| {
                            is_expanded.update(|v| *v = !*v);
                        })
                    >
                        {move || if is_expanded.get() { t!(i18n, common.collapse).into_any() } else { t!(i18n, common.expand).into_any() }}
                    </Button>
                </div>
            </Show>
            {move || {
                if is_expanded.get() {
                    view! {
                        <KanjiWritingSection kanji=kanji_for_animation.get_value() mode=KanjiViewMode::Frames />
                    }.into_any()
                } else {
                    ().into_any()
                }
            }}
            <CardHistoryModal
                is_open=Signal::derive(move || is_history_open.get())
                memory=memory_clone.clone()
                on_close=Callback::new(move |_| is_history_open.set(false))
            />
            <DeleteConfirmModal
                test_id="kanji-delete-modal"
                is_open=is_delete_modal_open
                is_deleting=is_deleting
                on_confirm=confirm_delete
                on_close=Callback::new(move |_| is_delete_modal_open.set(false))
            />
            <DrawingDrawer kanji=kanji_char.clone() is_open=drawing_drawer_open />
            <div class="border-t border-[var(--border-dark)] pt-2 mt-2 flex justify-between items-center">
                <Tag variant=Signal::derive(move || status.tag_variant())>
                    {status.label(&i18n)}
                </Tag>
                <div class="flex items-center gap-2">
                    <FavoriteButton
                        is_favorite=Signal::derive(move || is_favorite)
                        on_click=Callback::new(move |_| on_toggle_favorite.run(card_id))
                    />
                    <Show when=move || status != CardStatus::Learned>
                        <MarkAsKnownButton
                            on_click=Callback::new(move |_| on_mark_as_known.run(()))
                            test_id=Signal::derive(|| "kanji-card-item-mark-known-btn".to_string())
                        />
                    </Show>
                    <HistoryButton on_click=Callback::new(move |_| is_history_open.set(true)) />
                    <button
                        class="cursor-pointer transition-colors duration-200 hover:opacity-70"
                        on:click=move |_| drawing_drawer_open.set(true)
                        title=move || i18n.get_keys().kanji_page().practice_writing().inner().to_string()
                    >
                        <svg
                            xmlns="http://www.w3.org/2000/svg"
                            viewBox="0 0 24 24"
                            class="w-4 h-4"
                            fill="none"
                            stroke="currentColor"
                            stroke-width="2"
                            stroke-linecap="round"
                            stroke-linejoin="round"
                        >
                            <path d="M17 3a2.85 2.83 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5Z" />
                            <path d="m15 5 4 4" />
                        </svg>
                    </button>
                    <DeleteButton test_id="kanji-card-item-delete-btn" on_click=Callback::new(move |_| is_delete_modal_open.set(true)) />
                </div>
            </div>
        </Card>
    }
}
