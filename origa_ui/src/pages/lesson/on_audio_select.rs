use super::lesson_state::LessonState;
use leptos::prelude::*;
use origa::domain::{LessonCardView, Rating};

/// Answer handler for the AudioRecall card: `true` = «Знаю», `false` =
/// «Не знаю». Self-assessment maps binary to the classic ratings (Good /
/// Again) and advances immediately through the shared rating pipeline —
/// the same contract as the normal cards' RatingButtons (ADR-050). Unlike
/// the quiz/yesno/phrase feedback cards there is no verdict to read on
/// the answer side (the word and translation are already revealed), so a
/// separate «Далее» step would only repeat the same content.
pub fn create_on_audio_select(
    lesson_state: RwSignal<LessonState>,
    on_rate: Callback<Rating>,
) -> Callback<bool> {
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
        on_rate.run(rating);
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

    fn recorded_ratings() -> (RwSignal<Vec<Rating>>, Callback<Rating>) {
        let recorded = RwSignal::new(Vec::<Rating>::new());
        let recorder = recorded;
        let on_rate = Callback::new(move |rating: Rating| {
            recorder.update(|ratings| ratings.push(rating));
        });
        (recorded, on_rate)
    }

    #[test]
    fn audio_select_know_forwards_good_rating() {
        let state = Owner::new().with(|| {
            let (lesson_state, _) = setup_state(audio_recall_lesson_card());
            let (recorded, on_rate) = recorded_ratings();
            let on_audio_select = create_on_audio_select(lesson_state, on_rate);

            on_audio_select.run(true);
            recorded.get()
        });

        assert_eq!(state, vec![Rating::Good]);
    }

    #[test]
    fn audio_select_dont_know_forwards_again_rating() {
        let state = Owner::new().with(|| {
            let (lesson_state, _) = setup_state(audio_recall_lesson_card());
            let (recorded, on_rate) = recorded_ratings();
            let on_audio_select = create_on_audio_select(lesson_state, on_rate);

            on_audio_select.run(false);
            recorded.get()
        });

        assert_eq!(state, vec![Rating::Again]);
    }

    // The handler must be a no-op for any non-AudioRecall view: a wrong-mode
    // dispatch would otherwise arm a rating for a card that renders itself.
    #[test]
    fn audio_select_ignores_non_audio_recall_card() {
        let phrase_card = Card::Phrase(PhraseCard::new(Ulid::new()));
        let lesson_card = LessonCard::new(Ulid::new(), LessonCardView::Normal(phrase_card), false);

        let (ratings, waiting_armed) = Owner::new().with(|| {
            let (lesson_state, _) = setup_state(lesson_card);
            let (recorded, on_rate) = recorded_ratings();
            let on_audio_select = create_on_audio_select(lesson_state, on_rate);

            on_audio_select.run(true);
            (
                recorded.get_untracked(),
                lesson_state.get_untracked().waiting_for_next,
            )
        });

        assert!(
            ratings.is_empty(),
            "non-AudioRecall card must not produce a rating"
        );
        assert!(
            !waiting_armed,
            "audio handler must not arm the manual advance"
        );
    }
}
