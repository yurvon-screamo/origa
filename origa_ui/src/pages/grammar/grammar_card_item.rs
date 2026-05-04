use std::collections::HashSet;

use super::super::shared::{CardStatus, DeleteRequest};
use crate::i18n::use_i18n;
use crate::ui_components::{
    Card, CardActionBar, CardHistoryModal, DeleteConfirmModal, FuriganaText, Heading, HeadingLevel,
};
use leptos::prelude::*;
use origa::domain::{Card as DomainCard, NativeLanguage, StudyCard};
use ulid::Ulid;

#[component]
pub fn GrammarCardItem(
    study_card: StudyCard,
    #[prop(into)] native_language: Signal<NativeLanguage>,
    known_kanji: HashSet<String>,
    on_toggle_favorite: Callback<Ulid>,
    on_mark_as_known: Callback<()>,
    on_delete: Callback<DeleteRequest>,
    is_deleting: Signal<bool>,
    #[prop(into)] on_open_detail: Callback<()>,
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

    let study_card_for_title = study_card.clone();
    let title = Memo::new(move |_| {
        let lang = native_language.get();
        match study_card_for_title.card() {
            DomainCard::Grammar(grammar) => grammar
                .title(&lang)
                .ok()
                .map(|t| t.text().to_string())
                .unwrap_or_default(),
            _ => "?".to_string(),
        }
    });

    let status = CardStatus::from_study_card(&study_card);

    let status_tag_variant = Signal::derive(move || status.tag_variant());
    let status_label = Signal::derive(move || status.label(&i18n));
    let show_mark_as_known = Signal::derive(move || status != CardStatus::Learned);

    view! {
        <Card
            class="p-4 cursor-pointer h-full flex flex-col"
            test_id="grammar-card-item"
            on:click=move |_: leptos::ev::MouseEvent| on_open_detail.run(())
        >
            <div class="flex-1 min-h-0">
                <Heading level=HeadingLevel::H4 class="mb-1">
                    <FuriganaText text=title.get() known_kanji=known_kanji/>
                </Heading>
            </div>
            <div class="mt-auto shrink-0 pt-3">
                <CardActionBar
                    tag_variant=status_tag_variant
                    tag_label=status_label
                    is_favorite=Signal::derive(move || is_favorite)
                    on_toggle_favorite=Callback::new(move |_| on_toggle_favorite.run(card_id))
                    show_mark_as_known=show_mark_as_known
                    on_mark_as_known=Callback::new(move |_| on_mark_as_known.run(()))
                    on_history=Callback::new(move |_| is_history_open.set(true))
                    on_delete=Callback::new(move |_| is_delete_modal_open.set(true))
                    test_id=Signal::derive(|| "grammar-card-item".to_string())
                />
            </div>
        </Card>
        <CardHistoryModal
            is_open=Signal::derive(move || is_history_open.get())
            memory=memory_clone.clone()
            on_close=Callback::new(move |_| is_history_open.set(false))
        />
        <DeleteConfirmModal
            test_id="grammar-delete-modal"
            is_open=is_delete_modal_open
            is_deleting=is_deleting
            on_confirm=confirm_delete
            on_close=Callback::new(move |_| is_delete_modal_open.set(false))
        />
    }
}
