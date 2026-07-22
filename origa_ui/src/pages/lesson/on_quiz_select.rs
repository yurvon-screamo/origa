use super::lesson_state::LessonState;
use leptos::prelude::*;
use origa::domain::{LessonCardView, Rating};

pub fn create_on_quiz_select(lesson_state: RwSignal<LessonState>) -> Callback<usize> {
    // Defensive: without a dispose sentinel in context the handler returns a
    // no-op callback. The sentinel pattern is shared across all on_* handlers
    // for consistency with on_rate.rs (which additionally captures and checks
    // `is_disposed` inside its spawn_local — this handler is fully
    // synchronous, so only context existence is checked).
    if use_context::<StoredValue<()>>().is_none() {
        return Callback::new(move |_: usize| {});
    }

    Callback::new(move |option_index: usize| {
        lesson_state.update(|state| {
            state.selected_quiz_option = Some(option_index);
            state.showing_answer = true;
        });

        let Some(&card_id) = lesson_state
            .get()
            .card_ids
            .get(lesson_state.get().current_index)
        else {
            return;
        };

        if let Some(lesson_card) = lesson_state.get().cards.get(&card_id) {
            let is_correct = match lesson_card.view() {
                LessonCardView::Quiz(q) | LessonCardView::KanjiReadingQuiz(q) => {
                    Some(q.check_answer(option_index))
                },
                LessonCardView::GrammarQuiz(gq) => Some(gq.quiz().check_answer(option_index)),
                LessonCardView::PhraseListen { options, .. } => {
                    options.get(option_index).map(|o| o.is_correct())
                },
                _ => None,
            };

            if let Some(is_correct) = is_correct {
                let rating = if is_correct {
                    Rating::Good
                } else {
                    Rating::Again
                };

                // Pure-manual advance (ADR-033): the user dismisses the
                // feedback card themselves via Space/Enter/click. The previous
                // 1500ms timer was the source of the "stuck on the answer
                // window" complaint — replaced uniformly for phrase and
                // non-phrase quiz branches.
                lesson_state.update(|state| {
                    state.waiting_for_next = true;
                    state.pending_rating = Some(rating);
                });
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use origa::domain::{
        Card, LessonCard, LessonCardView, PhraseCard, QuizCard, QuizMode, QuizOption,
    };
    use ulid::Ulid;

    fn phrase_card() -> Card {
        Card::Phrase(PhraseCard::new(Ulid::new()))
    }

    /// Single-select quiz with 4 options, marking the one at `correct_index`.
    fn quiz_lesson_card(card: Card, correct_index: usize) -> LessonCard {
        let options: Vec<QuizOption> = (0..4)
            .map(|i| {
                QuizOption::new_simple(
                    if i == correct_index {
                        "correct".to_string()
                    } else {
                        "wrong".to_string()
                    },
                    i == correct_index,
                )
            })
            .collect();
        let quiz = QuizCard::new(card, options, QuizMode::Single);
        LessonCard::new(Ulid::new(), LessonCardView::Quiz(quiz), false)
    }

    fn setup_state(card: LessonCard) -> (RwSignal<LessonState>, Ulid) {
        let slot_id = Ulid::new();
        let mut cards = std::collections::HashMap::new();
        cards.insert(slot_id, card);
        let state = LessonState {
            card_ids: vec![slot_id],
            cards,
            ..LessonState::default()
        };
        (RwSignal::new(state), slot_id)
    }

    // Phrase branch sets waiting_for_next synchronously — no spawn_local on
    // this path, so the full post-condition is observable right after run().
    #[test]
    fn quiz_select_for_phrase_marks_showing_answer_and_waits_for_next() {
        let state = Owner::new().with(|| {
            provide_context(StoredValue::<()>::new(()));
            let card = quiz_lesson_card(phrase_card(), 0);
            let (lesson_state, _) = setup_state(card);
            let on_quiz_select = create_on_quiz_select(lesson_state);

            on_quiz_select.run(0);
            lesson_state.get()
        });

        assert!(state.showing_answer);
        assert_eq!(state.selected_quiz_option, Some(0));
        assert!(
            state.waiting_for_next,
            "phrase branch must set waiting_for_next synchronously"
        );
        assert_eq!(state.pending_rating, Some(Rating::Good));
    }

    // Non-phrase branch was previously gated by a 1500ms spawn_local timer;
    // pure-manual advance (ADR-033) unified it with the phrase branch. Both
    // branches now set `waiting_for_next` synchronously; only the synchronous
    // prefix is observable from native tests, which is why there is no
    // separate non-phrase characterization test — it would be redundant.

    // Without the dispose sentinel provided in context, the handler is
    // required to early-return a no-op callback.
    #[test]
    fn quiz_select_without_dispose_context_returns_noop_callback() {
        let state = Owner::new().with(|| {
            let card = quiz_lesson_card(phrase_card(), 0);
            let (lesson_state, _) = setup_state(card);
            let on_quiz_select = create_on_quiz_select(lesson_state);

            on_quiz_select.run(0);
            lesson_state.get()
        });

        assert!(!state.showing_answer, "noop callback must not mutate state");
        assert_eq!(state.selected_quiz_option, None);
    }
}
