use super::lesson_card::LessonCard;
use super::lesson_state::LessonContext;
use super::rating_buttons_view::RatingButtonsView;
use leptos::ev::KeyboardEvent;
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::application::srs_service::RateMode;
use origa::application::use_cases::{CompleteLessonUseCase, RateCardUseCase};
use origa::domain::Rating;
use origa::domain::User;
use origa::infrastructure::FsrsSrsService;

#[component]
pub fn LessonCardContainer() -> impl IntoView {
    let lesson_ctx = use_context::<LessonContext>().expect("lesson context");
    let current_user =
        use_context::<RwSignal<Option<User>>>().expect("current_user context not provided");
    let lesson_state = lesson_ctx.lesson_state;

    let show_answer = move || {
        lesson_state.update(|state| {
            state.showing_answer = true;
        });
    };

    let on_rate_callback = {
        let lesson_state = lesson_state;
        let current_user = current_user;
        let lesson_ctx = lesson_ctx.clone();

        Callback::new(move |rating: Rating| {
            let user = current_user.get();
            let state = lesson_state.get();

            if let (Some(user), Some(card_id)) = (user, state.card_ids.get(state.current_index)) {
                let card_id = *card_id;
                let user_id = user.id();
                let repo = lesson_ctx.repository.clone();
                let lesson_state = lesson_state;
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
        })
    };

    let handle_keydown = {
        let on_rate_callback = on_rate_callback;
        let lesson_ctx = lesson_ctx.clone();

        move |ev: KeyboardEvent| {
            let key = ev.key();
            let state = lesson_state.get();

            if lesson_ctx.is_completed.get() {
                return;
            }

            match key.as_str() {
                " " if !state.showing_answer => {
                    ev.prevent_default();
                    show_answer();
                }
                "1" if state.showing_answer => {
                    on_rate_callback.run(Rating::Again);
                }
                "2" if state.showing_answer => {
                    on_rate_callback.run(Rating::Hard);
                }
                "3" if state.showing_answer => {
                    on_rate_callback.run(Rating::Good);
                }
                "4" if state.showing_answer => {
                    on_rate_callback.run(Rating::Easy);
                }
                _ => {}
            }
        }
    };

    let current_card = Memo::new(move |_| {
        let state = lesson_state.get();
        state
            .card_ids
            .get(state.current_index)
            .and_then(|id| state.cards.get(id))
            .cloned()
    });

    view! {
        <div class="outline-none" tabindex="0" on:keydown=handle_keydown>
            <Show when=move || current_card.get().is_some()>
                <LessonCard
                    card=current_card.get().unwrap()
                    show_answer=lesson_state.get().showing_answer
                    on_show_answer=Callback::new(move |_| show_answer())
                />

                <Show when=move || lesson_state.get().showing_answer>
                    <RatingButtonsView on_rate=on_rate_callback />
                </Show>
            </Show>
        </div>
    }
}
