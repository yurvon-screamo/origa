use super::lesson_state::LessonContext;
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::domain::RateMode;
use origa::domain::Rating;
use origa::domain::User;
use origa::use_cases::RateCardUseCase;
use ulid::Ulid;

pub fn create_on_rate_callback(
    lesson_state: RwSignal<super::lesson_state::LessonState>,
    current_user: RwSignal<Option<User>>,
    lesson_ctx: LessonContext,
    is_rating: RwSignal<Option<Ulid>>,
) -> Callback<Rating> {
    Callback::new(move |rating: Rating| {
        let user = current_user.get_untracked();
        let state = lesson_state.get_untracked();

        if let (Some(user), Some(card_id)) = (user, state.card_ids.get(state.current_index)) {
            let card_id = *card_id;
            is_rating.set(Some(card_id));
            let user_id = user.id();
            let repo = lesson_ctx.repository.clone();
            let lesson_state = lesson_state;
            let is_completed = lesson_ctx.is_completed;
            let is_rating = is_rating;

            spawn_local(async move {
                let use_case = RateCardUseCase::new(&repo);

                let _ = use_case
                    .execute(user_id, card_id, RateMode::StandardLesson, rating)
                    .await;

                lesson_state.update(|state| {
                    let next_index = state.current_index + 1;
                    let total = state.card_ids.len();

                    state.review_count += 1;

                    if next_index >= total {
                        is_completed.set(true);
                    } else {
                        state.current_index = next_index;
                        state.showing_answer = false;
                        state.selected_quiz_option = None;
                    }
                });

                is_rating.set(None);
            });
        }
    })
}
