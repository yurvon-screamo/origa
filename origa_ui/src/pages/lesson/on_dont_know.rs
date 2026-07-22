use super::lesson_state::LessonState;
use leptos::prelude::*;
use origa::domain::Rating;

pub fn create_on_dont_know(lesson_state: RwSignal<LessonState>) -> Callback<()> {
    Callback::new(move |_: ()| {
        lesson_state.update(|state| {
            state.dont_know_selected = true;
            state.selected_quiz_option = None;
            state.selected_yesno_answer = None;
            state.showing_answer = true;
            state.selected_quiz_options.clear();
            state.multi_result = None;
            state.multi_quiz_submitted = true;
        });

        // Pure-manual advance (ADR-033): all branches converge on
        // waiting_for_next, regardless of card type or quiz mode. The
        // previous 1500ms timer branch for non-phrase non-multi was the
        // source of the "stuck on the answer window" complaint.
        lesson_state.update(|state| {
            state.waiting_for_next = true;
            state.pending_rating = Some(Rating::Again);
        });
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

    fn quiz_lesson_card(card: Card, mode: QuizMode) -> LessonCard {
        let options = vec![
            QuizOption::new_simple("a".to_string(), true),
            QuizOption::new_simple("b".to_string(), false),
        ];
        let quiz = QuizCard::new(card, options, mode);
        let view = match mode {
            QuizMode::Multi => LessonCardView::KanjiReadingQuiz(quiz),
            QuizMode::Single => LessonCardView::Quiz(quiz),
        };
        LessonCard::new(Ulid::new(), view, false)
    }

    fn setup_state(card: LessonCard) -> RwSignal<LessonState> {
        let slot_id = Ulid::new();
        let mut cards = std::collections::HashMap::new();
        cards.insert(slot_id, card);
        let state = LessonState {
            card_ids: vec![slot_id],
            cards,
            ..LessonState::default()
        };
        RwSignal::new(state)
    }

    // Phrase branch (and multi-quiz branch) set waiting_for_next synchronously
    // — no spawn_local on this path. Characterize to lock against regression
    // when Slice-2 unifies the non-phrase non-multi path.
    #[test]
    fn dont_know_for_phrase_sets_waiting_for_next_synchronously() {
        let state = Owner::new().with(|| {
            let card = quiz_lesson_card(phrase_card(), QuizMode::Single);
            let lesson_state = setup_state(card);
            let on_dont_know = create_on_dont_know(lesson_state);

            on_dont_know.run(());
            lesson_state.get()
        });

        assert!(state.showing_answer);
        assert!(state.dont_know_selected);
        assert!(
            state.waiting_for_next,
            "phrase branch must set waiting_for_next"
        );
        assert_eq!(state.pending_rating, Some(Rating::Again));
    }

    // Multi-quiz branch (KanjiReadingQuiz with mode=Multi) — same synchronous
    // waiting_for_next path as phrase, but reached by quiz shape, not card type.
    #[test]
    fn dont_know_for_multi_quiz_sets_waiting_for_next_synchronously() {
        let state = Owner::new().with(|| {
            // vocab card serialized via public serde repr — VocabularyCard::new
            // is #[cfg(test)] pub(crate) in the origa crate.
            let vocab: Card = serde_json::from_str(
                r#"{"Vocabulary":{"word":{"text":"test"},"reverse_side":null,"pos":null}}"#,
            )
            .expect("deserialize vocab card");
            let card = quiz_lesson_card(vocab, QuizMode::Multi);
            let lesson_state = setup_state(card);
            let on_dont_know = create_on_dont_know(lesson_state);

            on_dont_know.run(());
            lesson_state.get()
        });

        assert!(state.showing_answer);
        assert!(state.dont_know_selected);
        assert!(
            state.waiting_for_next,
            "multi-quiz branch must set waiting_for_next"
        );
        assert_eq!(state.pending_rating, Some(Rating::Again));
        assert!(
            state.multi_quiz_submitted,
            "multi-quiz branch must mark multi_quiz_submitted"
        );
        assert_eq!(
            state.multi_result, None,
            "dont_know never populates multi_result"
        );
    }
}
