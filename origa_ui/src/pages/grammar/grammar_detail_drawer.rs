use std::collections::HashSet;
use std::sync::Arc;

use super::super::shared::{CardStatus, DeleteRequest};
use super::grammar_practice_modal::GrammarPracticeModal;
use crate::i18n::{t, use_i18n};
use crate::ui_components::{
    CardActionBar, CardHistoryModal, DeleteConfirmModal, Drawer, FuriganaText, Heading,
    HeadingLevel, MarkdownText, Text, TextSize, TypographyVariant,
};
use leptos::prelude::*;
use origa::domain::{Card as DomainCard, NativeLanguage, StudyCard, User, get_rule_by_id};
use ulid::Ulid;

fn extract_grammar_rule(study_card: &StudyCard) -> Option<&'static origa::domain::GrammarRule> {
    match study_card.card() {
        DomainCard::Grammar(grammar) => get_rule_by_id(grammar.rule_id()),
        _ => None,
    }
}

#[component]
pub fn GrammarDetailDrawer(
    study_card: StudyCard,
    #[prop(into)] native_language: Signal<NativeLanguage>,
    known_kanji: HashSet<char>,
    user: Option<User>,
    on_toggle_favorite: Callback<Ulid>,
    on_mark_as_known: Callback<()>,
    on_delete: Callback<DeleteRequest>,
    is_deleting: Signal<bool>,
    #[prop(into)] on_close: Callback<()>,
) -> impl IntoView {
    let i18n = use_i18n();
    let card_id = *study_card.card_id();
    let is_favorite = study_card.is_favorite();
    let memory = study_card.memory();
    let memory_clone = memory.clone();

    let is_open = RwSignal::new(true);
    let is_history_open = RwSignal::new(false);
    let is_delete_modal_open = RwSignal::new(false);
    let is_practice_open = RwSignal::new(false);

    let confirm_delete = Callback::new(move |_| {
        on_delete.run(DeleteRequest {
            card_id,
            on_success: Callback::new(move |_| {
                is_delete_modal_open.set(false);
                is_open.set(false);
                on_close.run(());
            }),
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

    let title_signal = Signal::derive(move || title.get());

    let study_card_for_desc = study_card.clone();
    let description = Memo::new(move |_| {
        let lang = native_language.get();
        match study_card_for_desc.card() {
            DomainCard::Grammar(grammar) => grammar
                .description(&lang)
                .ok()
                .map(|d| d.text().to_string())
                .unwrap_or_default(),
            _ => "?".to_string(),
        }
    });

    let grammar_rule = extract_grammar_rule(&study_card);
    let has_quiz = grammar_rule.map(|r| r.has_format_map()).unwrap_or(false);
    let quiz_rule = grammar_rule;
    let quiz_user: StoredValue<Option<User>> = StoredValue::new(user);
    let known_kanji_stored: StoredValue<HashSet<char>> = StoredValue::new(known_kanji);

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

    let status = CardStatus::from_study_card(&study_card);
    let status_tag_variant = Signal::derive(move || status.tag_variant());
    let status_label = Signal::derive(move || status.label(&i18n));
    let show_mark_as_known = Signal::derive(move || status != CardStatus::Learned);

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

    let practice_label = t!(i18n, grammar_page.practice);
    let practice_fn: ChildrenFn = Arc::new(move || {
        let label = practice_label;
        view! {
            <Show when=move || quiz_user.get_value().is_some() && quiz_rule.is_some()>
                <button
                    class=move || if has_quiz {
                        "btn btn-olive text-sm cursor-pointer".to_string()
                    } else {
                        "btn btn-olive text-sm opacity-50 cursor-not-allowed".to_string()
                    }
                    disabled=!has_quiz
                    data-testid="grammar-card-practice-btn"
                    on:click=move |ev: leptos::ev::MouseEvent| {
                        ev.stop_propagation();
                        if has_quiz {
                            is_practice_open.set(true);
                        }
                    }
                >
                    {label}
                </button>
            </Show>
        }
        .into_any()
    });

    view! {
        <Drawer
            is_open=is_open
            on_close=Callback::new(move |_: leptos::ev::MouseEvent| {
                on_close.run(());
            })
            title=title_signal
            action_button=practice_fn
            test_id="grammar-detail-drawer"
        >
            <div class="space-y-4">
                <div class="border-b border-[var(--border-dark)] pb-3">
                    <Heading level=HeadingLevel::H4>
                        <FuriganaText text=title.get() known_kanji=known_kanji_stored.get_value()/>
                    </Heading>
                </div>

                <MarkdownText
                    content=Signal::derive(move || description.get())
                    known_kanji=known_kanji_stored.get_value()
                />

                <Text size=TextSize::Small variant=TypographyVariant::Muted>
                    {card_info_text}
                </Text>

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
        </Drawer>
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
        <Show when=move || is_practice_open.get() && quiz_rule.is_some() && quiz_user.get_value().is_some()>
            {move || {
                let rule = quiz_rule?;
                let user = quiz_user.get_value()?;
                Some(view! {
                    <GrammarPracticeModal
                        rule=rule
                        native_language=native_language
                        user=user
                        is_open=Signal::derive(move || is_practice_open.get())
                        on_close=Callback::new(move |_| is_practice_open.set(false))
                    />
                }.into_any())
            }}
        </Show>
    }
}
