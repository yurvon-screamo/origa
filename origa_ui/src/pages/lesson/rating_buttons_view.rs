use super::lesson_state::LessonContext;
use super::rating_buttons::RatingButtons;
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::application::srs_service::RateMode;
use origa::application::use_cases::{CompleteLessonUseCase, RateCardUseCase};
use origa::domain::{Rating, User};
use origa::infrastructure::FsrsSrsService;

#[component]
pub fn RatingButtonsView() -> impl IntoView {
    let current_user =
        use_context::<RwSignal<Option<User>>>().expect("current_user context not provided");
    let lesson_ctx = use_context::<LessonContext>().expect("lesson context");

    let on_rate = move |rating: Rating| {
        let user = current_user.get();
        let state = lesson_ctx.lesson_state.get();

        if let (Some(user), Some(card_id)) = (user, state.card_ids.get(state.current_index)) {
            let card_id = *card_id;
            let user_id = user.id();
            let repo = lesson_ctx.repository.clone();
            let lesson_state = lesson_ctx.lesson_state;
            let is_completed = lesson_ctx.is_completed;

            spawn_local(async move {
                let srs_service = match FsrsSrsService::new() {
                    Ok(s) => s,
                    Err(e) => {
                        web_sys::console::log_1(&format!("SRS error: {}", e).into());
                        return;
                    }
                };

                let use_case = RateCardUseCase::new(&repo, &srs_service);

                let _ = use_case
                    .execute(user_id, card_id, RateMode::StandardLesson, rating)
                    .await;

                lesson_state.update(|state| {
                    let next_index = state.current_index + 1;
                    let total = state.card_ids.len();

                    state.review_count += 1;

                    if next_index >= total {
                        let repo = repo.clone();

                        spawn_local(async move {
                            let use_case = CompleteLessonUseCase::new(&repo);
                            let _ = use_case
                                .execute(user_id, chrono::Duration::seconds(0))
                                .await;
                        });

                        is_completed.set(true);
                    } else {
                        state.current_index = next_index;
                        state.showing_answer = false;
                    }
                });
            });
        }
    };

    view! {
        <RatingButtons on_rate=Callback::new(on_rate) />
    }
}
