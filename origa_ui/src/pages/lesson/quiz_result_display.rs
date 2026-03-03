use crate::ui_components::{Text, TextSize};
use leptos::prelude::*;

use super::quiz_result::QuizResult;

#[component]
pub fn QuizResultDisplay(quiz_result: QuizResult) -> impl IntoView {
    view! {
        <div class="mt-6 text-center">
            <Text size=TextSize::Default class=move || {
                match quiz_result {
                    QuizResult::Correct => "text-[var(--success)] font-bold".to_string(),
                    QuizResult::Incorrect => "text-[var(--error)] font-bold".to_string(),
                    QuizResult::None => "".to_string(),
                }
            }>
                {move || match quiz_result {
                    QuizResult::Correct => "✓ Правильно!",
                    QuizResult::Incorrect => "✗ Неверно",
                    QuizResult::None => "",
                }}
            </Text>
        </div>
    }
}
