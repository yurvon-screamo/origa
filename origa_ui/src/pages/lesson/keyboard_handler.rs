use super::lesson_state::LessonContext;
use super::lesson_state::LessonState;
use leptos::ev::KeyboardEvent;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use origa::domain::{LessonCardView, QuizMode, Rating};
use ulid::Ulid;

pub struct KeyboardActions {
    pub on_rate: Callback<Rating>,
    pub on_quiz_select: Callback<usize>,
    pub on_yesno_select: Callback<bool>,
    pub on_quiz_dont_know: Callback<()>,
    pub on_yesno_dont_know: Callback<()>,
    pub on_quiz_toggle: Callback<usize>,
    pub on_quiz_submit: Callback<()>,
    pub show_answer: Box<dyn Fn()>,
    pub on_next_card: Callback<()>,
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

        // Приоритет: если ждём нажатия "Далее" — Space вызывает on_next_card
        if state.waiting_for_next && key == " " {
            ev.prevent_default();
            actions.on_next_card.run(());
            return;
        }

        let current_card_id = state.card_ids.get(state.current_index);
        let current_card = current_card_id.and_then(|id| state.cards.get(id));

        let is_multi_quiz = current_card
            .map(|c| {
                matches!(c.view(), LessonCardView::KanjiReadingQuiz(q) if q.mode() == QuizMode::Multi)
            })
            .unwrap_or(false);

        let is_quiz = current_card
            .map(|c| {
                matches!(
                    c.view(),
                    LessonCardView::Quiz(_)
                        | LessonCardView::KanjiReadingQuiz(_)
                        | LessonCardView::GrammarQuiz(_)
                )
            })
            .unwrap_or(false);
        let is_yesno = current_card
            .map(|c| matches!(c.view(), LessonCardView::YesNo(_)))
            .unwrap_or(false);
        let is_phrase_listen = current_card
            .map(|c| matches!(c.view(), LessonCardView::PhraseListen { .. }))
            .unwrap_or(false);
        if !state.showing_answer {
            if is_quiz || is_phrase_listen {
                if is_multi_quiz {
                    handle_multi_quiz_key(
                        &ev,
                        &key,
                        &actions.on_quiz_toggle,
                        &actions.on_quiz_submit,
                        &actions.on_quiz_dont_know,
                        !state.selected_quiz_options.is_empty(),
                    );
                } else {
                    handle_quiz_key(
                        &ev,
                        &key,
                        &actions.on_quiz_select,
                        &actions.on_quiz_dont_know,
                    );
                }
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

        if state.showing_answer && !is_quiz && !is_yesno && !is_phrase_listen {
            handle_rating_key(&key, &actions.on_rate);
            return;
        }

        if key == " " && !state.showing_answer && !is_quiz && !is_yesno && !is_phrase_listen {
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
    if let Some(index) = key.parse::<usize>().ok().filter(|&i| (1..=4).contains(&i)) {
        ev.prevent_default();
        on_select.run(index - 1);
        return;
    }

    if key == " " {
        ev.prevent_default();
        on_dont_know.run(());
    }
}

fn handle_multi_quiz_key(
    ev: &KeyboardEvent,
    key: &str,
    on_toggle: &Callback<usize>,
    on_submit: &Callback<()>,
    on_dont_know: &Callback<()>,
    has_selections: bool,
) {
    if let Some(index) = key.parse::<usize>().ok().filter(|&i| (1..=8).contains(&i)) {
        ev.prevent_default();
        on_toggle.run(index - 1);
        return;
    }

    match key {
        "Enter" => {
            ev.prevent_default();
            on_submit.run(());
        },
        " " => {
            ev.prevent_default();
            if has_selections {
                on_submit.run(());
            } else {
                on_dont_know.run(());
            }
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
            on_rate.run(Rating::Good);
        },
        _ => {},
    }
}

fn is_typing_target_kind(tag_name: &str, is_content_editable: bool) -> bool {
    tag_name.eq_ignore_ascii_case("INPUT")
        || tag_name.eq_ignore_ascii_case("TEXTAREA")
        || is_content_editable
}

pub(crate) fn is_typing_target(target: Option<&web_sys::EventTarget>) -> bool {
    let Some(el) = target.and_then(|t| t.dyn_ref::<web_sys::HtmlElement>()) else {
        return false;
    };
    is_typing_target_kind(&el.tag_name(), el.is_content_editable())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_and_textarea_are_typing_targets_case_insensitive() {
        assert!(is_typing_target_kind("INPUT", false));
        assert!(is_typing_target_kind("input", false));
        assert!(is_typing_target_kind("TEXTAREA", false));
        assert!(is_typing_target_kind("textarea", false));
    }

    #[test]
    fn contenteditable_is_typing_target_regardless_of_tag() {
        assert!(is_typing_target_kind("DIV", true));
        assert!(is_typing_target_kind("P", true));
        assert!(is_typing_target_kind("SPAN", true));
    }

    #[test]
    fn non_editable_buttons_and_divs_are_not_typing_targets() {
        assert!(!is_typing_target_kind("BUTTON", false));
        assert!(!is_typing_target_kind("DIV", false));
        assert!(!is_typing_target_kind("SPAN", false));
    }

    #[test]
    fn select_is_not_guarded() {
        assert!(!is_typing_target_kind("SELECT", false));
    }
}
