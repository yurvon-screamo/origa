use crate::i18n::*;
use crate::ui_components::{Text, TextSize};
use leptos::prelude::*;
use origa::domain::{MultiQuizResult, QuizOption};

use super::quiz_result::QuizResult;

#[component]
pub fn QuizResultDisplay(
    quiz_result: QuizResult,
    #[prop(default = None)] multi_result: Option<MultiQuizResult>,
    #[prop(default = vec![])] options: Vec<QuizOption>,
) -> impl IntoView {
    let i18n = use_i18n();

    let missed_texts: StoredValue<Vec<String>> = StoredValue::new(
        multi_result
            .as_ref()
            .map(|r| {
                r.missed
                    .iter()
                    .filter_map(|&idx| options.get(idx).map(|o| o.text().to_string()))
                    .collect()
            })
            .unwrap_or_default(),
    );

    let wrong_texts: StoredValue<Vec<String>> = StoredValue::new(
        multi_result
            .as_ref()
            .map(|r| {
                r.wrong_selections
                    .iter()
                    .filter_map(|&idx| options.get(idx).map(|o| o.text().to_string()))
                    .collect()
            })
            .unwrap_or_default(),
    );

    view! {
        <div class="mt-6 text-center">
            <Text size=TextSize::Default class=move || {
                match quiz_result {
                    QuizResult::Correct | QuizResult::MultiCorrect => {
                        "text-[var(--success)] font-bold".to_string()
                    },
                    QuizResult::Incorrect | QuizResult::MultiPartial => {
                        "text-[var(--error)] font-bold".to_string()
                    },
                    QuizResult::DontKnow => "text-[var(--fg-muted)] font-bold".to_string(),
                    QuizResult::None => "".to_string(),
                }
            }>
                {move || match quiz_result {
                    QuizResult::Correct => t!(i18n, lesson.correct).into_any(),
                    QuizResult::MultiCorrect => t!(i18n, lesson.multi_perfect).into_any(),
                    QuizResult::Incorrect => t!(i18n, lesson.incorrect).into_any(),
                    QuizResult::MultiPartial => t!(i18n, lesson.multi_partial).into_any(),
                    QuizResult::DontKnow => t!(i18n, lesson.dont_know_result).into_any(),
                    QuizResult::None => "".into_any(),
                }}
            </Text>

            <Show when=move || {
                quiz_result == QuizResult::MultiPartial && !missed_texts.get_value().is_empty()
            }>
                <div class="mt-2 text-sm text-[var(--fg-muted)]">
                    {t!(i18n, lesson.missed_options)} {" "}
                    <span class="font-semibold text-[var(--success)]">
                        {missed_texts.get_value().join(", ")}
                    </span>
                </div>
            </Show>

            <Show when=move || {
                quiz_result == QuizResult::MultiPartial && !wrong_texts.get_value().is_empty()
            }>
                <div class="mt-1 text-sm text-[var(--fg-muted)]">
                    {t!(i18n, lesson.extra_options)} {" "}
                    <span class="font-semibold text-[var(--error)]">
                        {wrong_texts.get_value().join(", ")}
                    </span>
                </div>
            </Show>
        </div>
    }
}
