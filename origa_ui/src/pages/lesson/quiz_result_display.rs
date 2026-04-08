use crate::i18n::*;
use crate::ui_components::{Text, TextSize};
use leptos::prelude::*;

use super::quiz_result::QuizResult;

#[component]
pub fn QuizResultDisplay(quiz_result: QuizResult) -> impl IntoView {
    let i18n = use_i18n();

    view! {
        <div class="mt-6 text-center">
            <Text size=TextSize::Default class=move || {
                match quiz_result {
                    QuizResult::Correct => "text-[var(--success)] font-bold".to_string(),
                    QuizResult::Incorrect => "text-[var(--error)] font-bold".to_string(),
                    QuizResult::DontKnow => "text-[var(--fg-muted)] font-bold".to_string(),
                    QuizResult::None => "".to_string(),
                }
            }>
                {move || match quiz_result {
                    QuizResult::Correct => t!(i18n, lesson.correct).into_any(),
                    QuizResult::Incorrect => t!(i18n, lesson.incorrect).into_any(),
                    QuizResult::DontKnow => t!(i18n, lesson.dont_know_result).into_any(),
                    QuizResult::None => "".into_any(),
                }}
            </Text>
        </div>
    }
}
