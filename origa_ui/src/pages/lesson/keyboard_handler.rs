use super::lesson_state::LessonContext;
use super::lesson_state::LessonState;
use leptos::ev::KeyboardEvent;
use leptos::prelude::*;
use origa::domain::{LessonCardView, Rating};
use ulid::Ulid;

pub struct KeyboardActions {
    pub on_rate: Callback<Rating>,
    pub on_quiz_select: Callback<usize>,
    pub on_yesno_select: Callback<bool>,
    pub on_quiz_dont_know: Callback<()>,
    pub on_yesno_dont_know: Callback<()>,
    pub show_answer: Box<dyn Fn()>,
}

pub fn create_keyboard_handler(
    lesson_ctx: LessonContext,
    is_rating: RwSignal<Option<Ulid>>,
    lesson_state: RwSignal<LessonState>,
    actions: KeyboardActions,
) -> impl Fn(KeyboardEvent) {
    move |ev: KeyboardEvent| {
        let key = ev.key();
        let state = lesson_state.get();

        if lesson_ctx.is_completed.get() || is_rating.get().is_some() {
            return;
        }

        let current_card_id = state.card_ids.get(state.current_index);
        let current_card = current_card_id.and_then(|id| state.cards.get(id));

        let is_quiz = matches!(current_card, Some(LessonCardView::Quiz(_)));
        let is_yesno = matches!(current_card, Some(LessonCardView::YesNo(_)));

        if !state.showing_answer {
            if is_quiz {
                handle_quiz_key(
                    &ev,
                    &key,
                    &actions.on_quiz_select,
                    &actions.on_quiz_dont_know,
                );
                return;
            }

            if is_yesno {
                handle_yesno_key(
                    &ev,
                    &key,
                    &actions.on_yesno_select,
                    &actions.on_yesno_dont_know,
                );
                return;
            }
        }

        if state.showing_answer && !is_quiz && !is_yesno {
            handle_rating_key(&key, &actions.on_rate);
            return;
        }

        if key == " " && !state.showing_answer && !is_quiz && !is_yesno {
            ev.prevent_default();
            (actions.show_answer)();
        }
    }
}

fn handle_quiz_key(
    ev: &KeyboardEvent,
    key: &str,
    on_select: &Callback<usize>,
    on_dont_know: &Callback<()>,
) {
    match key {
        "1" => {
            ev.prevent_default();
            on_select.run(0);
        },
        "2" => {
            ev.prevent_default();
            on_select.run(1);
        },
        "3" => {
            ev.prevent_default();
            on_select.run(2);
        },
        "4" => {
            ev.prevent_default();
            on_select.run(3);
        },
        " " => {
            ev.prevent_default();
            on_dont_know.run(());
        },
        _ => {},
    }
}

fn handle_yesno_key(
    ev: &KeyboardEvent,
    key: &str,
    on_select: &Callback<bool>,
    on_dont_know: &Callback<()>,
) {
    match key {
        "1" => {
            ev.prevent_default();
            on_select.run(false);
        },
        "2" => {
            ev.prevent_default();
            on_select.run(true);
        },
        " " => {
            ev.prevent_default();
            on_dont_know.run(());
        },
        _ => {},
    }
}

fn handle_rating_key(key: &str, on_rate: &Callback<Rating>) {
    match key {
        "1" => {
            on_rate.run(Rating::Again);
        },
        "2" => {
            on_rate.run(Rating::Hard);
        },
        "3" => {
            on_rate.run(Rating::Good);
        },
        "4" => {
            on_rate.run(Rating::Easy);
        },
        _ => {},
    }
}
