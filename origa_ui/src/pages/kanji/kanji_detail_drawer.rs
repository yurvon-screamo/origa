use std::collections::HashSet;

use super::super::shared::{CardStatus, DeleteRequest};
use crate::i18n::use_i18n;
use crate::ui_components::{
    CardActionBar, CardHistoryModal, DeleteConfirmModal, Drawer, KanjiDrawingPractice,
    KanjiViewMode, KanjiWritingSection, MarkdownText, Text, TextSize, TypographyVariant,
};
use leptos::prelude::*;
use origa::domain::{Card as DomainCard, NativeLanguage, StudyCard};
use ulid::Ulid;

#[component]
pub fn KanjiDetailDrawer(
    study_card: StudyCard,
    #[prop(into)] native_language: Signal<NativeLanguage>,
    known_kanji: HashSet<char>,
    on_toggle_favorite: Callback<Ulid>,
    on_mark_as_known: Callback<()>,
    on_delete: Callback<DeleteRequest>,
    is_deleting: Signal<bool>,
    on_close: Callback<()>,
) -> impl IntoView {
    let i18n = use_i18n();
    let card_id = *study_card.card_id();
    let is_favorite = study_card.is_favorite();
    let memory = study_card.memory().clone();
    let memory_history: StoredValue<origa::domain::MemoryHistory> =
        StoredValue::new(memory.clone());

    let is_drawer_open = RwSignal::new(true);
    let is_delete_modal_open = RwSignal::new(false);
    let is_history_open = RwSignal::new(false);

    let (kanji_char, radicals) = match study_card.card() {
        DomainCard::Kanji(kanji_card) => (
            kanji_card.kanji().text().to_string(),
            kanji_card.radicals_chars().into_iter().collect::<String>(),
        ),
        _ => ("?".to_string(), String::new()),
    };

    let status = CardStatus::from_study_card(&study_card);

    let confirm_delete = Callback::new(move |_| {
        on_delete.run(DeleteRequest {
            card_id,
            on_success: Callback::new(move |_| {
                is_delete_modal_open.set(false);
                is_drawer_open.set(false);
                on_close.run(());
            }),
        })
    });

    let study_card_for_desc = study_card.clone();
    let description = Memo::new(move |_| {
        let lang = native_language.get();
        match study_card_for_desc.card() {
            DomainCard::Kanji(kanji_card) => kanji_card
                .description(&lang)
                .ok()
                .map(|d| d.text().to_string())
                .unwrap_or_default(),
            _ => String::new(),
        }
    });

    let study_card_for_examples = study_card.clone();
    let example_words = Memo::new(move |_| {
        let lang = native_language.get();
        match study_card_for_examples.card() {
            DomainCard::Kanji(kanji_card) => kanji_card
                .example_words(&lang)
                .iter()
                .map(|w| (w.word().to_string(), w.meaning().to_string()))
                .collect::<Vec<_>>(),
            _ => Vec::new(),
        }
    });

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

    let has_radicals = !radicals.is_empty();
    let radicals_text = Signal::derive({
        let radicals = radicals.clone();
        move || {
            i18n.get_keys()
                .shared()
                .radicals_label()
                .inner()
                .to_string()
                .replacen("{}", &radicals, 1)
        }
    });

    let has_examples = Memo::new(move |_| !example_words.get().is_empty());
    let examples_heading = Signal::derive(move || {
        i18n.get_keys()
            .shared()
            .examples_label()
            .inner()
            .to_string()
            .replacen("{}", "", 1)
    });

    let card_info_text = Signal::derive(move || {
        i18n.get_keys()
            .shared()
            .card_info()
            .inner()
            .to_string()
            .replacen("{}", &next_review, 1)
            .replacen("{}", &difficulty, 1)
            .replacen("{}", &stability, 1)
    });

    let kanji_stored: StoredValue<String> = StoredValue::new(kanji_char.clone());

    view! {
        <Drawer
            is_open=is_drawer_open
            on_close=Callback::new(move |_: leptos::ev::MouseEvent| {
                on_close.run(());
            })
            title=Signal::derive(move || kanji_stored.get_value())
            test_id=Signal::derive(|| "kanji-detail-drawer".to_string())
        >
            <div class="space-y-4">
                <div class="flex justify-center py-4">
                    <span class="text-5xl font-serif">{kanji_stored.get_value()}</span>
                </div>

                <MarkdownText
                    content=Signal::derive(move || description.get())
                    known_kanji=known_kanji.clone()
                />

                <Show when=move || has_radicals>
                    <Text size=TextSize::Small variant=TypographyVariant::Muted>
                        {radicals_text}
                    </Text>
                </Show>

                <Show when=move || has_examples.get()>
                    <div>
                        <Text size=TextSize::Small variant=TypographyVariant::Primary class=Signal::derive(|| "mb-1".to_string())>
                            {examples_heading}
                        </Text>
                        <div class="space-y-1">
                            <For
                                each=move || example_words.get()
                                key=|(word, _)| word.clone()
                                children=move |(word, meaning): (String, String)| {
                                    view! {
                                        <Text size=TextSize::Small variant=TypographyVariant::Muted>
                                            {format!("{} — {}", word, meaning)}
                                        </Text>
                                    }
                                }
                            />
                        </div>
                    </div>
                </Show>

                <KanjiWritingSection
                    kanji=kanji_stored.get_value()
                    mode=KanjiViewMode::Frames
                />

                <KanjiDrawingPractice kanji=kanji_stored.get_value() />

                <Text size=TextSize::Small variant=TypographyVariant::Muted>
                    {card_info_text}
                </Text>

                <CardActionBar
                    tag_variant=Signal::derive(move || status.tag_variant())
                    tag_label=Signal::derive(move || status.label(&i18n))
                    is_favorite=Signal::derive(move || is_favorite)
                    on_toggle_favorite=Callback::new(move |_| on_toggle_favorite.run(card_id))
                    show_mark_as_known=Signal::derive(move || status != CardStatus::Learned)
                    on_mark_as_known=Callback::new(move |_| on_mark_as_known.run(()))
                    on_history=Callback::new(move |_| is_history_open.set(true))
                    on_delete=Callback::new(move |_| is_delete_modal_open.set(true))
                    test_id=Signal::derive(|| "kanji-detail-drawer-action".to_string())
                />
            </div>
        </Drawer>
        <CardHistoryModal
            is_open=Signal::derive(move || is_history_open.get())
            memory=memory_history.get_value()
            on_close=Callback::new(move |_| is_history_open.set(false))
        />
        <DeleteConfirmModal
            test_id="kanji-delete-modal"
            is_open=is_delete_modal_open
            is_deleting=is_deleting
            on_confirm=confirm_delete
            on_close=Callback::new(move |_| is_delete_modal_open.set(false))
        />
    }
}
