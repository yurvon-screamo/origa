use super::keyboard_handler::create_keyboard_handler;
use super::lesson_card::LessonCard;
use super::lesson_state::LessonContext;
use super::on_quiz_select::create_on_quiz_select;
use super::on_rate::create_on_rate_callback;
use super::quiz_card::QuizCardView;
use super::rating_buttons_view::RatingButtonsView;
use leptos::prelude::*;
use origa::domain::User;
use origa::domain::{GrammarInfo, LessonCardView, Rating};
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

    let container_ref = NodeRef::<leptos::html::Div>::new();

    Effect::new(move |_| {
        if let Some(el) = container_ref.get() {
            let _ = el.focus();
        }
    });

    view! {
        <div class="outline-none" tabindex="0" node_ref=container_ref on:keydown=handle_keydown>
            <Show when=move || current_card_view.get().is_some()>
                <Show when=move || !is_quiz_mode.get()>
                    {move || {
                        current_card_view.get().map(|card_view| {
                            render_lesson_card(
                                card_view,
                                lesson_state.get().showing_answer,
                                Callback::new(move |_| show_answer()),
                                on_rate_callback,
                                is_rating,
                            )
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

struct LessonCardParams {
    card: origa::domain::Card,
    is_reversed: bool,
    grammar_info: Option<GrammarInfo>,
}

fn render_lesson_card(
    card_view: LessonCardView,
    show_answer: bool,
    on_show_answer: Callback<()>,
    on_rate_callback: Callback<Rating>,
    is_rating: RwSignal<Option<Ulid>>,
) -> impl IntoView {
    let params = match card_view {
        LessonCardView::Normal(card) => LessonCardParams {
            card,
            is_reversed: false,
            grammar_info: None,
        },
        LessonCardView::Reversed(card) => LessonCardParams {
            card,
            is_reversed: true,
            grammar_info: None,
        },
        LessonCardView::GrammarMutated { card, grammar_info } => LessonCardParams {
            card,
            is_reversed: false,
            grammar_info: Some(grammar_info),
        },
        LessonCardView::Quiz(_) => {
            return view! { <div/> }.into_any();
        }
    };

    view! {
        <LessonCard
            card=params.card
            is_reversed=params.is_reversed
            show_answer=show_answer
            on_show_answer=on_show_answer
            grammar_info=params.grammar_info
        />

        <Show when=move || show_answer>
            <RatingButtonsView
                on_rate=on_rate_callback
                disabled=Signal::derive(move || is_rating.get().is_some())
            />
        </Show>
    }
    .into_any()
}
