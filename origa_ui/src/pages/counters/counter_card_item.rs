use super::super::shared::{CardStatus, DeleteRequest, format_answer_text};
use crate::i18n::use_i18n;
use crate::ui_components::{CardActionBar, DeleteConfirmModal, FsrsMetrics, Tag, TagVariant};
use leptos::prelude::*;
use leptos_router::components::A;
use origa::domain::{Card as DomainCard, NativeLanguage, StudyCard};
use ulid::Ulid;

#[component]
pub fn CounterCardItem(
    study_card: StudyCard,
    #[prop(into)] native_language: Signal<NativeLanguage>,
    on_toggle_favorite: Callback<Ulid>,
    on_mark_as_known: Callback<Ulid>,
    on_delete: Callback<DeleteRequest>,
    is_deleting: Signal<bool>,
) -> impl IntoView {
    let i18n = use_i18n();
    let card_id = *study_card.card_id();
    let is_favorite = study_card.is_favorite();
    let memory = study_card.memory();

    let is_delete_modal_open = RwSignal::new(false);

    let confirm_delete = Callback::new(move |_| {
        on_delete.run(DeleteRequest {
            card_id,
            on_success: Callback::new(move |_| is_delete_modal_open.set(false)),
        })
    });

    let (suffix, bindings_summary) = match study_card.card() {
        DomainCard::Counter(counter_card) => {
            let known = counter_card
                .bindings()
                .iter()
                .filter(|b| b.memory().is_known_card())
                .count();
            (
                counter_card.suffix().to_string(),
                format!("{known}/{}", counter_card.bindings().len()),
            )
        },
        _ => ("?".to_string(), String::new()),
    };

    let study_card_for_answer = study_card.clone();
    let answer_text = Memo::new(move |_| {
        let lang = native_language.get();
        format_answer_text(study_card_for_answer.card(), &lang)
    });

    let bindings_label = Signal::derive({
        let summary = bindings_summary.clone();
        move || {
            i18n.get_keys()
                .counters()
                .bindings_learned()
                .inner()
                .to_string()
                .replacen("{}", &summary, 1)
        }
    });

    let status = CardStatus::from_study_card(&study_card);
    let show_mark_as_known = status != CardStatus::Learned;

    view! {
        <div class="counter-card anima-lift" data-testid="counter-card-item">
            <div class="counter-card-badge">
                <Tag variant=Signal::derive(move || status.tag_variant())>
                    {move || status.label(&i18n)}
                </Tag>
            </div>
            <A href=format!("/counters/{card_id}") attr:class="counter-card-link">
                <div class="counter-card-char-box">
                    <span class="counter-card-char">{suffix.clone()}</span>
                </div>
                <div class="counter-card-content">
                    <Show when=move || !answer_text.get().is_empty()>
                        <span class="counter-card-answer">{move || answer_text.get()}</span>
                    </Show>
                    <Show when=move || !bindings_summary.is_empty()>
                        <span class="counter-card-bindings" data-testid="counter-card-bindings">
                            {move || bindings_label.get()}
                        </span>
                    </Show>
                </div>
            </A>
            <div class="counter-card-divider"></div>
            <div class="counter-card-footer">
                <FsrsMetrics
                    difficulty=memory.difficulty().map(|d| d.value())
                    stability=memory.stability().map(|s| s.value())
                    test_id=Signal::derive(|| "counter-card-fsrs".to_string())
                />
                <div class="counter-card-actions">
                    <CardActionBar
                        tag_variant=TagVariant::default()
                        tag_label=Signal::derive(|| "".to_string())
                        is_favorite=Signal::derive(move || is_favorite)
                        on_toggle_favorite=Callback::new(move |_| on_toggle_favorite.run(card_id))
                        show_mark_as_known=Signal::derive(move || show_mark_as_known)
                        on_mark_as_known=Callback::new(move |_| on_mark_as_known.run(card_id))
                        on_delete=Callback::new(move |_| is_delete_modal_open.set(true))
                        test_id=Signal::derive(|| "counter-card-item".to_string())
                        show_tag=Signal::derive(|| false)
                    />
                </div>
            </div>
        </div>
        <DeleteConfirmModal
            test_id="counter-delete-modal"
            is_open=is_delete_modal_open
            is_deleting=is_deleting
            on_confirm=confirm_delete
            on_close=Callback::new(move |_| is_delete_modal_open.set(false))
        />
    }
}
