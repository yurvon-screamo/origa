use super::lesson_state::LessonState;
use leptos::prelude::*;
use origa::domain::{LessonCardView, Rating};

/// Answer handler for the AudioRecall card: `true` = «Знаю», `false` =
/// «Не знаю». Self-assessment maps binary to the classic ratings (Good /
/// Again) and defers the advance to the user via `waiting_for_next`
/// (pure-manual advance, ADR-033) — the same contract as on_yesno_select.
pub fn create_on_audio_select(lesson_state: RwSignal<LessonState>) -> Callback<bool> {
    // Defensive: without a dispose sentinel in context the handler returns a
    // no-op callback. Same pattern as on_quiz_select / on_yesno_select.
    if use_context::<StoredValue<()>>().is_none() {
        return Callback::new(move |_: bool| {});
    }

    Callback::new(move |knows_word: bool| {
        let state = lesson_state.get();
        let Some(card_id) = state.card_ids.get(state.current_index) else {
            return;
        };
        let Some(lesson_card) = state.cards.get(card_id) else {
            return;
        };
        if !matches!(lesson_card.view(), LessonCardView::AudioRecall(_)) {
            return;
        }

        let rating = if knows_word {
            Rating::Good
        } else {
            Rating::Again
        };

        lesson_state.update(|state| {
            state.showing_answer = true;
            state.waiting_for_next = true;
            state.pending_rating = Some(rating);
        });
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use origa::domain::{Card, LessonCard, PhraseCard};
    use std::collections::HashMap;
    use ulid::Ulid;

    fn audio_recall_lesson_card() -> LessonCard {
        // VocabularyCard::new is #[cfg(test)] pub(crate) in the origa crate,
        // so round-trip a vocab card through its public serde representation
        // (same approach as on_rate.rs tests).
        let card: Card = serde_json::from_str(
            r#"{"Vocabulary":{"word":{"text":"温度"},"reverse_side":null,"pos":null}}"#,
        )
        .expect("deserialize vocab card");
        LessonCard::new(Ulid::new(), LessonCardView::AudioRecall(card), false)
    }

    fn setup_state(card: LessonCard) -> (RwSignal<LessonState>, Ulid) {
        let slot_id = Ulid::new();
        let mut cards = HashMap::new();
        cards.insert(slot_id, card);
        let state = LessonState {
            card_ids: vec![slot_id],
            cards,
            ..LessonState::default()
        };
        (RwSignal::new(state), slot_id)
    }

    #[test]
    fn audio_select_know_reveals_answer_and_sets_good_rating() {
        let state = Owner::new().with(|| {
            provide_context(StoredValue::<()>::new(()));
            let (lesson_state, _) = setup_state(audio_recall_lesson_card());
            let on_audio_select = create_on_audio_select(lesson_state);

            on_audio_select.run(true);
            lesson_state.get()
        });

        assert!(state.showing_answer, "answer side must be revealed");
        assert!(state.waiting_for_next, "manual advance must be armed");
        assert_eq!(state.pending_rating, Some(Rating::Good));
    }

    #[test]
    fn audio_select_dont_know_sets_again_rating() {
        let state = Owner::new().with(|| {
            provide_context(StoredValue::<()>::new(()));
            let (lesson_state, _) = setup_state(audio_recall_lesson_card());
            let on_audio_select = create_on_audio_select(lesson_state);

            on_audio_select.run(false);
            lesson_state.get()
        });

        assert!(state.showing_answer);
        assert!(state.waiting_for_next);
        assert_eq!(state.pending_rating, Some(Rating::Again));
    }

    // The handler must be a no-op for any non-AudioRecall view: a wrong-mode
    // dispatch would otherwise arm a rating for a card that renders itself.
    #[test]
    fn audio_select_ignores_non_audio_recall_card() {
        let phrase_card = Card::Phrase(PhraseCard::new(Ulid::new()));
        let lesson_card = LessonCard::new(Ulid::new(), LessonCardView::Normal(phrase_card), false);

        let state = Owner::new().with(|| {
            provide_context(StoredValue::<()>::new(()));
            let (lesson_state, _) = setup_state(lesson_card);
            let on_audio_select = create_on_audio_select(lesson_state);

            on_audio_select.run(true);
            lesson_state.get()
        });

        assert!(
            !state.showing_answer && state.pending_rating.is_none(),
            "non-AudioRecall card must not be mutated by the audio handler"
        );
    }

    #[test]
    fn audio_select_without_dispose_context_returns_noop_callback() {
        let state = Owner::new().with(|| {
            let (lesson_state, _) = setup_state(audio_recall_lesson_card());
            let on_audio_select = create_on_audio_select(lesson_state);

            on_audio_select.run(true);
            lesson_state.get()
        });

        assert!(!state.showing_answer, "noop callback must not mutate state");
    }
}
