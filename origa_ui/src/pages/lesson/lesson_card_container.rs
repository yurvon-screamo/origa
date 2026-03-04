use super::keyboard_handler::create_keyboard_handler;
use super::lesson_card::LessonCard;
use super::lesson_state::LessonContext;
use super::on_quiz_select::create_on_quiz_select;
use super::on_rate::create_on_rate_callback;
use super::quiz_card::QuizCardView;
use super::rating_buttons_view::RatingButtonsView;
use leptos::ev::HTMLElement;
use leptos::prelude::*;
use origa::domain::LessonCardView;
use origa::domain::User;
use ulid::Ulid;
use wasm_bindgen::JsCast;

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

    let on_rate_callback =
        create_on_rate_callback(lesson_state, current_user, lesson_ctx.clone(), is_rating);

    let on_quiz_select = create_on_quiz_select(lesson_state, on_rate_callback);

    let handle_keydown = create_keyboard_handler(
        lesson_ctx.clone(),
        is_rating,
        on_rate_callback,
        on_quiz_select,
        lesson_state,
        show_answer,
    );

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
                            match &card_view {
                                LessonCardView::Normal(card)
                                | LessonCardView::Reversed(card)
                                | LessonCardView::GrammarMutated(card) => {
                                    let is_reversed = matches!(card_view, LessonCardView::Reversed(_));
                                    let card = card.clone();

                                    Some(view! {
                                        <LessonCard
                                            card=card
                                            is_reversed=is_reversed
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
                                }
                                _ => None,
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
