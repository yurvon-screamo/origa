use super::acquaintance_state::{AcquaintanceContext, AcquaintanceStage};
use super::acquaintance_view::AcquaintanceHeaderStrip;
use super::lesson_progress::LessonProgress;
use super::lesson_state::LessonContext;
use crate::feedback::{FeedbackCategory, FeedbackContext, FeedbackSource, FeedbackSubject};
use crate::i18n::use_i18n;
use leptos::prelude::*;
use leptos_icons::Icon;
use leptos_router::hooks::use_navigate;
use origa::domain::{Card, CardAnswer, LessonCardView, NativeLanguage};

use super::lesson_state::LessonState;

#[component]
pub fn LessonHeader() -> impl IntoView {
    let i18n = use_i18n();
    let navigate = use_navigate();
    let lesson_ctx = use_context::<LessonContext>().expect("LessonContext not provided");
    let feedback = use_context::<FeedbackContext>();
    let is_muted = lesson_ctx.is_muted;
    let lesson_state = lesson_ctx.lesson_state;
    let core_count = lesson_ctx.core_count;

    // Во время руки знакомства место LessonProgress занимает полоса руки:
    // прогресс живёт в общем хедере и не съедает полезную высоту карточки
    // (баг-репорт: пустой хедер + полоса над контентом).
    let hand_active = use_context::<AcquaintanceContext>().map(|acq| {
        Signal::derive(move || {
            acq.state
                .with(|state| state.stage != AcquaintanceStage::Inactive && state.hand.is_some())
        })
    });
    let show_lesson_progress = move || {
        hand_active
            .as_ref()
            .map(|signal| !signal.get())
            .unwrap_or(true)
    };

    let toggle_mute = move || {
        is_muted.update(|m| *m = !*m);
    };

    let current = Signal::derive(move || lesson_state.get().current_index + 1);
    let total = Signal::derive(move || lesson_state.get().card_ids.len());
    let core_count_signal = Signal::derive(move || core_count.get());

    // Report entry (ADR-055): enabled only while the answer is shown — the
    // subject is built from the revealed answer, and on the question side a
    // report modal would spoil it. `disabled` (not hidden) keeps the header
    // layout stable across card phases. Two answer sources: regular lesson
    // cards (`LessonState.showing_answer`) and the acquaintance hand training
    // (`AcquaintanceContext.showing_answer`).
    let acq_context = use_context::<AcquaintanceContext>();
    let acq_showing_answer = acq_context.as_ref().map(|acq| acq.showing_answer);
    let acq_current_card = acq_context.as_ref().map(|acq| acq.current_card);
    let can_report = Signal::derive(move || {
        if lesson_state.get().showing_answer {
            return true;
        }
        acq_showing_answer
            .as_ref()
            .is_some_and(|signal| signal.get())
    });
    let on_report = Callback::new(move |()| {
        let Some(feedback) = feedback.clone() else {
            return;
        };
        let state = lesson_state.get_untracked();
        let lang = lesson_ctx.native_language.get_untracked();
        // During the acquaintance hand the subject comes from the hand slides
        // (the hand stores ids, not domain cards); regular cards come from
        // the lesson state.
        let current_id = acq_current_card
            .as_ref()
            .and_then(|signal| signal.get_untracked());
        let subject = match (&acq_context, current_id) {
            (Some(acq), Some(card_id)) => acquaintance_slide_subject(acq, &card_id),
            _ => lesson_feedback_subject(&state, &lang),
        };
        if let Some(subject) = subject {
            let ui_language = i18n.get_locale().to_string();
            feedback.open(
                FeedbackCategory::Content,
                FeedbackSource::LessonHeader,
                subject,
                &ui_language,
            );
        } else {
            tracing::warn!(
                current_index = state.current_index,
                "Lesson report clicked with no resolvable subject"
            );
        }
    });

    view! {
        <div class="flex items-center gap-2 mb-2 shrink-0" data-testid="lesson-header">
            <button
                data-testid="lesson-back-btn"
                class="flex items-center gap-1 text-sm text-muted-foreground hover:text-foreground transition-colors shrink-0 cursor-pointer"
                aria-label={move || i18n.get_keys().common().back().inner().to_string()}
                on:click=move |_| navigate("/home", Default::default())
            >
                <Icon icon=icondata::LuArrowLeft width="14" height="14" />
            </button>

            <div class="flex-1 min-w-0">
                <Show when=show_lesson_progress fallback=move || view! { <AcquaintanceHeaderStrip /> }>
                    <LessonProgress current=current total=total core_count=core_count_signal />
                </Show>
            </div>

            <button
                data-testid="lesson-mute-btn"
                class="p-1.5 text-muted-foreground hover:text-foreground transition-colors shrink-0 cursor-pointer"
                data-muted=move || if is_muted.get() { "true" } else { "false" }
                on:click=move |_| toggle_mute()
            >
                {move || if is_muted.get() {
                    view! { <Icon icon=icondata::LuVolumeX width="16" height="16" /> }
                        .into_any()
                } else {
                    view! { <Icon icon=icondata::LuVolume2 width="16" height="16" /> }
                        .into_any()
                }}
            </button>
            <button
                data-testid="lesson-report-btn"
                class="lesson-report-btn shrink-0"
                disabled=move || !can_report.get()
                aria-disabled=move || (!can_report.get()).to_string()
                aria-label=move || {
                    if can_report.get() {
                        i18n.get_keys().feedback().lesson_report_aria().inner().to_string()
                    } else {
                        i18n.get_keys().feedback().lesson_report_disabled_aria().inner().to_string()
                    }
                }
                on:click=move |_| on_report.run(())
            >
                <Icon icon=icondata::LuTriangleAlert width="16" height="16" />
            </button>
        </div>
    }
}

/// Subject of the current acquaintance-hand training card, built from the
/// hand slide (the hand itself stores only ids + types). Mirrors what the
/// training card shows: the Japanese surface and, after the reveal, the
/// translation/description as the context line.
fn acquaintance_slide_subject(
    acq: &AcquaintanceContext,
    card_id: &ulid::Ulid,
) -> Option<FeedbackSubject> {
    use super::acquaintance_state::AcquaintanceSlideData;

    let slide = acq
        .slides
        .get_untracked()
        .into_iter()
        .find(|slide| &slide.card_id() == card_id)?;
    let (surface, context_line) = match &slide {
        AcquaintanceSlideData::Vocabulary {
            word, translations, ..
        } => (word.clone(), translations.join(", ")),
        AcquaintanceSlideData::Kanji { kanji, name, .. } => (kanji.clone(), name.clone()),
        AcquaintanceSlideData::Grammar {
            title,
            short_description,
            ..
        } => (title.clone(), short_description.clone()),
    };
    Some(FeedbackSubject {
        surface,
        reading: None,
        context_line: (!context_line.is_empty()).then_some(context_line),
    })
}

/// Build the feedback subject from a base card: the question surface plus
/// the revealed answer as the context line.
fn feedback_subject_for(base: &Card, lang: &NativeLanguage) -> Option<FeedbackSubject> {
    let surface = base.question(lang).ok()?.text().to_string();
    let answer_line = base
        .answer(lang)
        .map(|answer: CardAnswer| answer.text_projection())
        .unwrap_or_default();
    Some(FeedbackSubject {
        surface,
        reading: None,
        context_line: (!answer_line.is_empty()).then_some(answer_line),
    })
}

/// Build the feedback subject from the current lesson card: the question
/// surface plus the revealed answer as the context line. Returns `None` when
/// there is no current card (empty lesson edge case).
fn lesson_feedback_subject(state: &LessonState, lang: &NativeLanguage) -> Option<FeedbackSubject> {
    let card = state
        .card_ids
        .get(state.current_index)
        .and_then(|id| state.cards.get(id))?;
    feedback_subject_for(base_lesson_card(card.view())?, lang)
}

/// Every `LessonCardView` variant carries a base [`Card`]; extract it without
/// cloning.
fn base_lesson_card(view: &LessonCardView) -> Option<&Card> {
    Some(match view {
        LessonCardView::Normal(card)
        | LessonCardView::Reversed(card)
        | LessonCardView::Writing(card)
        | LessonCardView::AudioRecall(card)
        | LessonCardView::GrammarMutated { card, .. }
        | LessonCardView::PhraseListen { card, .. } => card,
        LessonCardView::Quiz(quiz) | LessonCardView::KanjiReadingQuiz(quiz) => quiz.card(),
        LessonCardView::YesNo(yesno) => yesno.card(),
        LessonCardView::GrammarQuiz(grammar_quiz) => grammar_quiz.card(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use origa::domain::LessonCard;
    use std::collections::HashMap;
    use ulid::Ulid;

    fn vocab_card(word: &str) -> Card {
        serde_json::from_str(&format!(
            r#"{{"Vocabulary":{{"word":{{"text":"{word}"}},"reverse_side":null,"pos":null}}}}"#
        ))
        .expect("deserialize vocab card fixture")
    }

    fn state_with_current_card(view: LessonCardView) -> LessonState {
        let slot = Ulid::new();
        let mut cards = HashMap::new();
        cards.insert(slot, LessonCard::new(slot, view, false));
        LessonState {
            card_ids: vec![slot],
            cards,
            ..LessonState::default()
        }
    }

    /// The subject of a normal vocabulary card: surface = question word,
    /// context = answer translations (any-card answer works — here none are
    /// serialized, so the line is empty and omitted).
    #[test]
    fn feedback_subject_of_normal_card_uses_question_surface() {
        let state = state_with_current_card(LessonCardView::Normal(vocab_card("温度")));
        let subject =
            lesson_feedback_subject(&state, &NativeLanguage::English).expect("subject built");
        assert_eq!(subject.surface, "温度");
    }

    /// Quiz variants wrap the base card — the subject must unwrap it.
    #[test]
    fn feedback_subject_unwraps_quiz_variant() {
        let card = vocab_card("温度");
        let quiz = LessonCardView::Quiz(origa::domain::QuizCard::new(
            card,
            vec![],
            origa::domain::QuizMode::default(),
        ));
        let state = state_with_current_card(quiz);
        let subject =
            lesson_feedback_subject(&state, &NativeLanguage::English).expect("subject built");
        assert_eq!(subject.surface, "温度");
    }

    /// No current card (empty lesson) → no subject, no panic.
    #[test]
    fn feedback_subject_none_for_empty_lesson() {
        let state = LessonState::default();
        assert!(lesson_feedback_subject(&state, &NativeLanguage::English).is_none());
    }
}
