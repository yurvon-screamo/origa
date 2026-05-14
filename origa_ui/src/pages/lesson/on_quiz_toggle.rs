use super::lesson_state::LessonState;
use leptos::prelude::*;

pub fn create_on_quiz_toggle(lesson_state: RwSignal<LessonState>) -> Callback<usize> {
    Callback::new(move |option_index: usize| {
        let state = lesson_state.get();
        if state.showing_answer {
            return;
        }

        lesson_state.update(|state| {
            if state.selected_quiz_options.contains(&option_index) {
                state.selected_quiz_options.remove(&option_index);
            } else {
                state.selected_quiz_options.insert(option_index);
            }
        });
    })
}
