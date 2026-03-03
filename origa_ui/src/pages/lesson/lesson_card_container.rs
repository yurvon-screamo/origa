use super::lesson_card::LessonCard;
use super::lesson_state::LessonContext;
use super::quiz_card::QuizCardView;
use super::rating_buttons_view::RatingButtonsView;
use leptos::ev::KeyboardEvent;
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::application::srs_service::RateMode;
use origa::application::use_cases::RateCardUseCase;
use origa::domain::User;
use origa::domain::{LessonCardView, Rating};
use origa::infrastructure::FsrsSrsService;
use ulid::Ulid;

#[component]
pub fn LessonCardContainer() -> impl IntoView {
    let lesson_ctx = use_context::<LessonContext>().expect("lesson context");
    let current_user =
        use_context::<RwSignal<Option<User>>>().expect("current_user context not provided");
    let lesson_state = lesson_ctx.lesson_state;
    let is_rating = RwSignal::new(None::<Ulid>);

    let show_answer = move || {
        lesson_state.update(|state| {
            state.showing_answer = true;
        });
    };

    let on_rate_callback = {
        let lesson_state = lesson_state;
        let current_user = current_user;
        let lesson_ctx = lesson_ctx.clone();
        let is_rating = is_rating;

        Callback::new(move |rating: Rating| {
            let user = current_user.get();
            let state = lesson_state.get();

            if let (Some(user), Some(card_id)) = (user, state.card_ids.get(state.current_index)) {
                let card_id = *card_id;
                is_rating.set(Some(card_id));
                let user_id = user.id();
                let repo = lesson_ctx.repository.clone();
                let lesson_state = lesson_state;
                let is_completed = lesson_ctx.is_completed;
                let is_rating = is_rating;

                spawn_local(async move {
                    let srs_service = match FsrsSrsService::new() {
                        Ok(s) => s,
                        Err(e) => {
                            web_sys::console::error_1(&format!("SRS error: {}", e).into());
                            is_rating.set(None);
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
    };

    let on_quiz_select = {
        let lesson_state = lesson_state;
        let on_rate = on_rate_callback;

        Callback::new(move |option_index: usize| {
            lesson_state.update(|state| {
                state.selected_quiz_option = Some(option_index);
                state.showing_answer = true;
            });

            if let Some(card_view) = lesson_state.get().cards.get(
                lesson_state
                    .get()
                    .card_ids
                    .get(lesson_state.get().current_index)
                    .unwrap(),
            ) && let LessonCardView::Quiz(quiz) = card_view
            {
                let is_correct = quiz.check_answer(option_index);
                let rating = if is_correct {
                    Rating::Good
                } else {
                    Rating::Hard
                };

                let on_rate_clone = on_rate.clone();
                spawn_local(async move {
                    gloo_timers::future::TimeoutFuture::new(1500).await;
                    on_rate_clone.run(rating);
                });
            }
        })
    };

    let handle_keydown = {
        let on_rate_callback = on_rate_callback;
        let lesson_ctx = lesson_ctx.clone();
        let is_rating = is_rating;
        let on_quiz_select = on_quiz_select.clone();

        move |ev: KeyboardEvent| {
            let key = ev.key();
            let state = lesson_state.get();

            if lesson_ctx.is_completed.get() || is_rating.get().is_some() {
                return;
            }

            let is_quiz = matches!(
                state.cards.get(state.card_ids.get(state.current_index).unwrap()),
                Some(LessonCardView::Quiz(_))
            );

            if is_quiz && !state.showing_answer {
                match key.as_str() {
                    "1" => {
                        ev.prevent_default();
                        on_quiz_select.run(0);
                    }
                    "2" => {
                        ev.prevent_default();
                        on_quiz_select.run(1);
                    }
                    "3" => {
                        ev.prevent_default();
                        on_quiz_select.run(2);
                    }
                    "4" => {
                        ev.prevent_default();
                        on_quiz_select.run(3);
                    }
                    _ => {}
                }
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

    let current_card_view = Memo::new(move |_| {
        let state = lesson_state.get();
        state
            .card_ids
            .get(state.current_index)
            .and_then(|id| state.cards.get(id))
            .cloned()
    });

    let is_quiz_mode = Memo::new(move |_| {
        current_card_view
            .get()
            .map(|view| matches!(view, LessonCardView::Quiz(_)))
            .unwrap_or(false)
    });

    view! {
        <div class="outline-none" tabindex="0" on:keydown=handle_keydown>
            <Show when=move || current_card_view.get().is_some()>
                <Show when=move || !is_quiz_mode.get()>
                    {move || {
                        current_card_view.get().and_then(|card_view| {
                            if let LessonCardView::Normal(card) = card_view {
                                Some(view! {
                                    <LessonCard
                                        card=card
                                        show_answer=lesson_state.get().showing_answer
                                        on_show_answer=Callback::new(move |_| show_answer())
                                    />

                                    <Show when=move || lesson_state.get().showing_answer>
                                        <RatingButtonsView
                                            on_rate=on_rate_callback
                                            disabled=Signal::derive(move || is_rating.get().is_some())
                                        />
                                    </Show>
                                })
                            } else {
                                None
                            }
                        })
                    }}
                </Show>

                <Show when=move || is_quiz_mode.get()>
                    {move || {
                        current_card_view.get().and_then(|card_view| {
                            if let LessonCardView::Quiz(quiz) = card_view {
                                let selected_option = lesson_state.get().selected_quiz_option;
                                let show_result = lesson_state.get().showing_answer;

                                Some(view! {
                                    <QuizCardView
                                        quiz_card=quiz
                                        show_result=show_result
                                        selected_option=selected_option
                                        on_select_option=on_quiz_select
                                    />
                                })
                            } else {
                                None
                            }
                        })
                    }}
                </Show>
            </Show>
        </div>
    }
}
