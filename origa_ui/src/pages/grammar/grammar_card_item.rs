use std::collections::HashSet;

use super::super::shared::{CardStatus, DeleteRequest};
use crate::i18n::use_i18n;
use crate::ui_components::{
    CardActionBar, CardHistoryModal, DeleteConfirmModal, FsrsMetrics, FuriganaText, Tag, TagVariant,
};
use leptos::prelude::*;
use leptos_router::components::A;
use origa::domain::{Card as DomainCard, NativeLanguage, StudyCard};
use ulid::Ulid;

#[component]
pub fn GrammarCardItem(
    study_card: StudyCard,
    #[prop(into)] native_language: Signal<NativeLanguage>,
    known_kanji: HashSet<char>,
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

    let study_card_for_answer = study_card.clone();
    let answer_text = Memo::new(move |_| {
        let lang = native_language.get();
        study_card_for_answer
            .card()
            .answer(&lang)
            .map(|a| a.text().to_string())
            .unwrap_or_default()
    });

    let status = CardStatus::from_study_card(&study_card);
    let show_mark_as_known = status != CardStatus::Learned;

    view! {
        <div class="grammar-card anima-lift" data-testid="grammar-card-item">
            <div class="grammar-card-badge">
                <Tag variant=Signal::derive(move || status.tag_variant())>
                    {move || status.label(&i18n)}
                </Tag>
            </div>
            <A href=format!("/grammar/{}", card_id) attr:class="grammar-card-link">
                <div class="grammar-card-rule-box">
                    <FuriganaText text=title.get() known_kanji=known_kanji/>
                </div>
                <div class="grammar-card-content">
                    <Show when=move || !answer_text.get().is_empty()>
                        <span class="grammar-card-answer">{move || answer_text.get()}</span>
                    </Show>
                </div>
            </A>
            <div class="grammar-card-divider"></div>
            <div class="grammar-card-footer">
                <FsrsMetrics
                    difficulty=memory.difficulty().map(|d| d.value())
                    stability=memory.stability().map(|s| s.value())
                    test_id=Signal::derive(|| "grammar-card-fsrs".to_string())
                />
                <div class="grammar-card-actions">
                    <CardActionBar
                        tag_variant=TagVariant::default()
                        tag_label=Signal::derive(|| "".to_string())
                        is_favorite=Signal::derive(move || is_favorite)
                        on_toggle_favorite=Callback::new(move |_| on_toggle_favorite.run(card_id))
                        show_mark_as_known=Signal::derive(move || show_mark_as_known)
                        on_mark_as_known=Callback::new(move |_| on_mark_as_known.run(()))
                        on_history=Callback::new(move |_| is_history_open.set(true))
                        on_delete=Callback::new(move |_| is_delete_modal_open.set(true))
                        test_id=Signal::derive(|| "grammar-card-item".to_string())
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
            test_id="grammar-delete-modal"
            is_open=is_delete_modal_open
            is_deleting=is_deleting
            on_confirm=confirm_delete
            on_close=Callback::new(move |_| is_delete_modal_open.set(false))
        />
    }
}
