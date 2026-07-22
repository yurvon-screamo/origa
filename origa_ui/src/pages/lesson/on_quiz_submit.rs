use super::lesson_state::LessonState;
use leptos::prelude::*;
use origa::domain::LessonCardView;

pub fn create_on_quiz_submit(lesson_state: RwSignal<LessonState>) -> Callback<()> {
    Callback::new(move |_: ()| {
        let state = lesson_state.get();
        let selected: Vec<usize> = state.selected_quiz_options.iter().copied().collect();

        if selected.is_empty() {
            return;
        }

        let card_id = state.card_ids.get(state.current_index).copied();

        let Some(card_id) = card_id else {
            return;
        };

        let Some(lesson_card) = state.cards.get(&card_id) else {
            return;
        };

        let quiz = match lesson_card.view() {
            LessonCardView::KanjiReadingQuiz(q) => q,
            _ => return,
        };

        let multi_result = quiz.check_multi_answers(&selected);
        let rating = multi_result.rating_lenient();

        lesson_state.update(|state| {
            state.showing_answer = true;
            state.multi_quiz_submitted = true;
            state.multi_result = Some(multi_result);
        });

        // Pure-manual advance (ADR-033): the user dismisses the feedback card
        // themselves via Space/Enter/click. Replaces the previous 1500ms timer
        // branch for non-phrase non-multi KanjiReadingQuiz submissions.
        lesson_state.update(|state| {
            state.waiting_for_next = true;
            state.pending_rating = Some(rating);
        });
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use origa::domain::{Card, LessonCard, LessonCardView, QuizCard, QuizMode, QuizOption};
    use std::collections::HashSet;
    use ulid::Ulid;

    fn vocab_card() -> Card {
        serde_json::from_str(
            r#"{"Vocabulary":{"word":{"text":"test"},"reverse_side":null,"pos":null}}"#,
        )
        .expect("deserialize vocab card")
    }

    /// Multi-quiz requires `LessonCardView::KanjiReadingQuiz` with `QuizMode::Multi`.
    fn multi_quiz_lesson_card(card: Card) -> LessonCard {
        let options = vec![
            QuizOption::new_simple("a".to_string(), true),
            QuizOption::new_simple("b".to_string(), false),
        ];
        let quiz = QuizCard::new(card, options, QuizMode::Multi);
        LessonCard::new(Ulid::new(), LessonCardView::KanjiReadingQuiz(quiz), false)
    }

    fn setup_state_with_selections(
        card: LessonCard,
        selections: &[usize],
    ) -> RwSignal<LessonState> {
        let slot_id = Ulid::new();
        let mut cards = std::collections::HashMap::new();
        cards.insert(slot_id, card);
        let state = LessonState {
            card_ids: vec![slot_id],
            cards,
            selected_quiz_options: selections.iter().copied().collect::<HashSet<_>>(),
            ..LessonState::default()
        };
        RwSignal::new(state)
    }

    // Multi-quiz submit always lands on the synchronous waiting_for_next
    // branch (because `is_multi_quiz == true`), regardless of card type.
    // This is the behavior Slice-2 must NOT change.
    #[test]
    fn quiz_submit_for_multi_quiz_sets_waiting_for_next_synchronously() {
        let state = Owner::new().with(|| {
            let card = multi_quiz_lesson_card(vocab_card());
            let lesson_state = setup_state_with_selections(card, &[0]);
            let on_submit = create_on_quiz_submit(lesson_state);

            on_submit.run(());
            lesson_state.get()
        });

        assert!(state.showing_answer);
        assert!(state.multi_quiz_submitted);
        assert!(
            state.multi_result.is_some(),
            "multi_result must be populated by submit"
        );
        assert!(
            state.waiting_for_next,
            "multi-quiz submit must set waiting_for_next"
        );
        assert!(
            state.pending_rating.is_some(),
            "multi-quiz submit must set pending_rating"
        );
    }

    // Defensive: empty selection is a no-op (return early).
    #[test]
    fn quiz_submit_with_empty_selection_is_noop() {
        let state = Owner::new().with(|| {
            let card = multi_quiz_lesson_card(vocab_card());
            let lesson_state = setup_state_with_selections(card, &[]);
            let on_submit = create_on_quiz_submit(lesson_state);

            on_submit.run(());
            lesson_state.get()
        });

        assert!(
            !state.showing_answer,
            "empty selection must not mutate state"
        );
        assert!(!state.multi_quiz_submitted);
        assert!(state.multi_result.is_none());
    }

    // Defensive: KanjiReadingQuiz is the only view accepted; other quiz views
    // (e.g. plain Quiz) early-return without mutating state.
    #[test]
    fn quiz_submit_for_non_kanji_reading_quiz_view_is_noop() {
        let state = Owner::new().with(|| {
            // Build a plain Quiz view with multi mode — still rejected, since
            // the matcher accepts only KanjiReadingQuiz.
            let options = vec![
                QuizOption::new_simple("a".to_string(), true),
                QuizOption::new_simple("b".to_string(), false),
            ];
            let quiz = QuizCard::new(vocab_card(), options, QuizMode::Multi);
            let card = LessonCard::new(Ulid::new(), LessonCardView::Quiz(quiz), false);
            let lesson_state = setup_state_with_selections(card, &[0]);
            let on_submit = create_on_quiz_submit(lesson_state);

            on_submit.run(());
            lesson_state.get()
        });

        assert!(
            !state.showing_answer,
            "non-KanjiReadingQuiz view must be rejected"
        );
    }
}
